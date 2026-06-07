use super::client::JkClient;
use super::crypto::make_card_password;
use super::token_cache::{clear_cached_token, get_token, should_clear_cached_token};
use super::PayResult;
use redis::aio::ConnectionLike;
use serde_json::{json, Value};
use std::time::Instant;

const SELLER_ID: &str = "248933040709";
const ID_TYPE: &str = "1";
const PAY_CHANNEL: i64 = 6;

const ITEM_CODE: &str = "PAJKPOS1169888";
const ITEM_NAME: &str = "中医-其他";
const ITEM_CATEGORY: &str = "D04";
const ITEM_CATEGORY_DESP: &str = "保健服务";
const ITEM_GMT_MODIFIED: &str = "2025-10-29 16:56:44";

pub(super) fn calc_jk_payment_amount_fen(total_amount_fen: i64) -> i64 {
    ((total_amount_fen as f64) / 0.95).round() as i64
}

fn parse_money_value(value: Option<&Value>) -> Option<f64> {
    let value = value?;
    if let Some(n) = value.as_f64() {
        return Some(n);
    }
    if let Some(n) = value.as_i64() {
        return Some(n as f64);
    }
    if let Some(n) = value.as_u64() {
        return Some(n as f64);
    }
    value.as_str().and_then(|s| s.trim().parse::<f64>().ok())
}

pub(super) fn extract_first_money_field(value: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| parse_money_value(value.get(*key)))
}

fn extract_first_string_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(|v| v.as_str()))
        .map(|s| s.trim().to_string())
}

fn extract_channel_name(value: &Value) -> Option<String> {
    extract_first_string_field(
        value,
        &[
            "payChannelName",
            "channelName",
            "accountName",
            "name",
            "payChannelDesc",
            "description",
            "cardName",
        ],
    )
}

pub(super) fn choose_health_card_balance(channels: &[Value]) -> Option<f64> {
    let mut best: Option<(bool, bool, f64)> = None;

    for ch in channels {
        let Some(balance) = extract_first_money_field(
            ch,
            &[
                "balance",
                "availableBalance",
                "amount",
                "balanceAmount",
                "remainAmount",
            ],
        ) else {
            continue;
        };
        let positive = balance > 0.01;
        let name = extract_channel_name(ch).unwrap_or_default();
        let looks_like_health_card =
            name.contains("健康卡") || name.contains("养老险") || name.contains("养老保险");

        let candidate = (positive, looks_like_health_card, balance);
        let should_replace = best
            .map(|curr| {
                candidate.0 > curr.0
                    || (candidate.0 == curr.0
                        && (candidate.1 > curr.1
                            || (candidate.1 == curr.1 && candidate.2 > curr.2)))
            })
            .unwrap_or(true);

        if should_replace {
            best = Some(candidate);
        }
    }

    best.map(|(_, _, balance)| balance)
}

fn simplify_error_message(raw_msg: &str) -> String {
    let msg = raw_msg.to_lowercase();

    if msg.contains("交易密码错误") || msg.contains("密码错误") {
        if msg.contains("超限") {
            return "密码错误次数过多，请稍后再试".to_string();
        }
        return "支付密码错误".to_string();
    }

    if msg.contains("余额不足") || msg.contains("账户余额") {
        return "余额不足".to_string();
    }

    if msg.contains("卡状态") || msg.contains("卡片状态") {
        return "健康卡状态异常，请联系客服".to_string();
    }

    if msg.contains("金额") && msg.contains("小于") {
        return "支付金额过小".to_string();
    }

    if msg.contains("验证失败") {
        return "健康卡验证失败".to_string();
    }

    if let Some(idx) = raw_msg.find("试算失败：") {
        return raw_msg[idx + "试算失败：".len()..].to_string();
    }
    if let Some(idx) = raw_msg.find("预结算失败：") {
        return raw_msg[idx + "预结算失败：".len()..].to_string();
    }
    if let Some(idx) = raw_msg.find('：') {
        return raw_msg[idx + '：'.len_utf8()..].to_string();
    }

    raw_msg.to_string()
}

pub(super) fn build_trade_line(amount_yuan: f64) -> Value {
    json!({
        "itemCode": ITEM_CODE,
        "itemName": ITEM_NAME,
        "category": ITEM_CATEGORY,
        "categoryDesp": ITEM_CATEGORY_DESP,
        "spec": "", "brand": "", "itemType": "",
        "itemBarCode": "", "itemApprovalNumber": "",
        "itemSubCategory": "", "itemManufacturer": "",
        "gmtModified": ITEM_GMT_MODIFIED,
        "lastModifier": "",
        "qty": 1,
        "price": amount_yuan,
        "amount": amount_yuan,
        "approvalNumber": "",
        "barcode": "",
        "manufacturer": "",
        "subCategory": "",
    })
}

fn extract_order_no(data: &Value) -> Result<String, String> {
    let content = data
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| format!("获取订单号失败: {}", data))?;

    content
        .get("orderNo")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("获取订单号失败: {}", data))
}

async fn fetch_order_no(jk: &JkClient, wtk: &str) -> Result<Value, String> {
    jk.api(
        "baize.getStoreAndOrderNo",
        &[("req", "{}"), ("sellerId", SELLER_ID)],
        wtk,
    )
    .await
}

async fn query_payment_channel(
    jk: &JkClient,
    wtk: &str,
    card_no: &str,
    card_password: &str,
) -> Result<(String, Option<f64>), String> {
    let enc_pwd = if card_password.trim().is_empty() {
        String::new()
    } else {
        make_card_password(card_no, card_password)?
    };

    let mut channel_req = json!({
        "idType": ID_TYPE,
        "cardNo": card_no,
        "queryBalance": true,
    });
    if !enc_pwd.is_empty() {
        channel_req["password"] = json!(enc_pwd);
    }
    let channel_req_str =
        serde_json::to_string(&channel_req).map_err(|e| format!("json error: {e}"))?;

    let data = jk
        .api(
            "baize.queryPayChannelByEntityCard",
            &[("req", &channel_req_str), ("sellerId", SELLER_ID)],
            wtk,
        )
        .await?;

    let ch_content = data
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| {
            data.pointer("/stat/stateList")
                .and_then(|v| v.as_str())
                .unwrap_or("健康卡验证失败")
                .to_string()
        })?;

    if ch_content.get("success").and_then(|v| v.as_bool()) != Some(true) {
        let msg = ch_content
            .get("returnMsg")
            .and_then(|v| v.as_str())
            .unwrap_or("健康卡验证失败")
            .to_string();
        return Err(simplify_error_message(&msg));
    }

    let balance = ch_content
        .get("payChannels")
        .and_then(|v| v.as_array())
        .and_then(|channels| choose_health_card_balance(channels))
        .or_else(|| {
            extract_first_money_field(
                ch_content,
                &[
                    "balance",
                    "availableBalance",
                    "amount",
                    "balanceAmount",
                    "remainAmount",
                ],
            )
        });

    Ok((enc_pwd, balance))
}

async fn precalc_payment(
    jk: &JkClient,
    wtk: &str,
    card_no: &str,
    enc_pwd: &str,
    order_no: &str,
    amount_yuan: f64,
    line: &Value,
) -> Result<Value, String> {
    let precalc_req = json!({
        "cardNo": card_no,
        "password": enc_pwd,
        "idType": ID_TYPE,
        "xrefNo": order_no,
        "amount": amount_yuan,
        "lines": [line],
        "payChannel": PAY_CHANNEL,
    });
    let precalc_req_str =
        serde_json::to_string(&precalc_req).map_err(|e| format!("json error: {e}"))?;

    let data = jk
        .api(
            "baize.drugCardPreCalc",
            &[("req", &precalc_req_str), ("sellerId", SELLER_ID)],
            wtk,
        )
        .await?;

    let pr = data
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| format!("预结算失败: {}", data))?;

    if pr.get("success").and_then(|v| v.as_bool()) != Some(true) {
        let msg = pr
            .get("returnMsg")
            .and_then(|v| v.as_str())
            .unwrap_or("预结算失败");
        return Err(simplify_error_message(msg));
    }

    if pr.get("fundAmount").is_none() || pr["fundAmount"].is_null() {
        let msg = pr
            .get("returnMsg")
            .and_then(|v| v.as_str())
            .unwrap_or("预结算业务失败");
        return Err(simplify_error_message(msg));
    }

    Ok(pr.clone())
}

async fn poll_ready_plan_if_needed(jk: &JkClient, wtk: &str, order_no: &str) -> Result<(), String> {
    let poll_req = json!({"xrefNo": order_no, "payChannel": PAY_CHANNEL});
    let poll_req_str = serde_json::to_string(&poll_req).map_err(|e| format!("json error: {e}"))?;

    for i in 0..15 {
        let poll_started = Instant::now();
        let poll_data = jk
            .api(
                "baize.pollReadyPlan",
                &[("req", &poll_req_str), ("sellerId", SELLER_ID)],
                wtk,
            )
            .await?;
        tracing::info!(
            "[JK Pay] pollReadyPlan {} elapsed_ms={}",
            i + 1,
            poll_started.elapsed().as_millis()
        );

        let pr2 = poll_data
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|a| a.first())
            .cloned()
            .unwrap_or(json!({}));
        tracing::info!(
            "[JK Pay] pollReadyPlan {}: finish={:?}",
            i + 1,
            pr2.get("finish")
        );

        if pr2.get("finish").and_then(|v| v.as_bool()) == Some(true) {
            return Ok(());
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}

async fn execute_payment(
    jk: &JkClient,
    wtk: &str,
    order_no: &str,
    amount_yuan: f64,
    line: &Value,
) -> Result<Value, String> {
    let pay_req = json!({
        "idType": ID_TYPE,
        "xrefNo": order_no,
        "amount": amount_yuan,
        "lines": [line],
        "payChannel": PAY_CHANNEL,
    });
    let pay_req_str = serde_json::to_string(&pay_req).map_err(|e| format!("json error: {e}"))?;

    jk.api(
        "baize.drugCardPay",
        &[("req", &pay_req_str), ("sellerId", SELLER_ID)],
        wtk,
    )
    .await
}

async fn try_pay_with_password(
    jk: &JkClient,
    wtk: &str,
    card_no: &str,
    card_password: &str,
    order_no: &str,
    amount_yuan: f64,
    line: &Value,
) -> Result<PayResult, String> {
    let (enc_pwd, _channel_balance) =
        query_payment_channel(jk, wtk, card_no, card_password).await?;

    let precalc_started = Instant::now();
    let pr = precalc_payment(jk, wtk, card_no, &enc_pwd, order_no, amount_yuan, line).await?;
    tracing::info!(
        "[JK Pay] drugCardPreCalc elapsed_ms={}",
        precalc_started.elapsed().as_millis()
    );

    if pr.get("pollReadyPlan").and_then(|v| v.as_bool()) == Some(true) {
        poll_ready_plan_if_needed(jk, wtk, order_no).await?;
    }

    let precalc_account_amount = extract_first_money_field(
        &pr,
        &["fundAmount", "deductAmount", "accountAmount", "payAmount"],
    )
    .unwrap_or(0.0);
    let precalc_cash_amount =
        extract_first_money_field(&pr, &["cashAmount", "cashPayAmount", "selfPayAmount"])
            .unwrap_or(0.0);
    if precalc_cash_amount > 0.01 {
        return Err(format!(
            "第三方返回现金补差，已拒绝支付：账户支付 {:.2} 元，现金支付 {:.2} 元，应付 {:.2} 元",
            precalc_account_amount, precalc_cash_amount, amount_yuan
        ));
    }
    if precalc_account_amount + 0.01 < amount_yuan {
        return Err(format!(
            "健康卡余额不足：账户支付 {:.2} 元，应付 {:.2} 元",
            precalc_account_amount, amount_yuan
        ));
    }

    let pay_started = Instant::now();
    let pay_data = execute_payment(jk, wtk, order_no, amount_yuan, line).await?;
    tracing::info!(
        "[JK Pay] drugCardPay elapsed_ms={}",
        pay_started.elapsed().as_millis()
    );
    let r = pay_data
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .cloned()
        .unwrap_or(json!({}));
    let deduct_amount = r
        .get("deductAmount")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let cash_amount =
        extract_first_money_field(&r, &["cashAmount", "cashPayAmount", "selfPayAmount"])
            .unwrap_or(0.0);
    let pay_success = r.get("success").and_then(|v| v.as_bool()) == Some(true)
        && deduct_amount + 0.01 >= amount_yuan
        && cash_amount <= 0.01;

    if pay_success {
        let paid_amount_fen = (deduct_amount * 100.0).round() as i64;

        Ok(PayResult {
            success: true,
            paid_amount: paid_amount_fen,
            order_status: None,
            external_order_no: Some(order_no.to_string()),
            fail_reason: None,
        })
    } else {
        let msg = if cash_amount > 0.01 {
            format!(
                "第三方返回现金补差，已拒绝确认：账户支付 {:.2} 元，现金支付 {:.2} 元，应付 {:.2} 元",
                deduct_amount, cash_amount, amount_yuan
            )
        } else if deduct_amount + 0.01 < amount_yuan {
            format!(
                "健康卡余额不足：账户支付 {:.2} 元，应付 {:.2} 元",
                deduct_amount, amount_yuan
            )
        } else {
            r.get("returnMsg")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    pay_data
                        .pointer("/stat/stateList")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "支付失败".to_string())
        };

        Err(msg)
    }
}

pub(super) async fn do_jk_pay<C>(
    redis: &mut C,
    seller_username: &str,
    seller_password: &str,
    card_no: &str,
    card_password: &str,
    total_amount_fen: i64,
) -> Result<PayResult, String>
where
    C: ConnectionLike + Send,
{
    let deduct_amount_fen = calc_jk_payment_amount_fen(total_amount_fen);
    do_jk_pay_with_deduct_amount(
        redis,
        seller_username,
        seller_password,
        card_no,
        card_password,
        deduct_amount_fen,
        Some(total_amount_fen),
    )
    .await
}

pub(super) async fn do_jk_pay_exact_amount<C>(
    redis: &mut C,
    seller_username: &str,
    seller_password: &str,
    card_no: &str,
    card_password: &str,
    deduct_amount_fen: i64,
) -> Result<PayResult, String>
where
    C: ConnectionLike + Send,
{
    do_jk_pay_with_deduct_amount(
        redis,
        seller_username,
        seller_password,
        card_no,
        card_password,
        deduct_amount_fen,
        None,
    )
    .await
}

async fn do_jk_pay_with_deduct_amount<C>(
    redis: &mut C,
    seller_username: &str,
    seller_password: &str,
    card_no: &str,
    card_password: &str,
    deduct_amount_fen: i64,
    source_amount_fen: Option<i64>,
) -> Result<PayResult, String>
where
    C: ConnectionLike + Send,
{
    if deduct_amount_fen <= 0 {
        return Err("payment amount must be positive".to_string());
    }

    let amount_yuan = deduct_amount_fen as f64 / 100.0;
    let started = Instant::now();

    if let Some(source_amount_fen) = source_amount_fen {
        tracing::info!(
            "[JK Pay] source_amount_fen={} deduct_amount_fen={} amount_yuan={}",
            source_amount_fen,
            deduct_amount_fen,
            amount_yuan
        );
    } else {
        tracing::info!(
            "[JK Pay] exact deduct_amount_fen={} amount_yuan={}",
            deduct_amount_fen,
            amount_yuan
        );
    }

    let jk = JkClient::new()?;
    let line = build_trade_line(amount_yuan);

    for attempt in 0..2u32 {
        let token_started = Instant::now();
        let wtk = get_token(redis, &jk, seller_username, seller_password).await?;
        tracing::info!(
            "[JK Pay] get_token attempt={} elapsed_ms={}",
            attempt + 1,
            token_started.elapsed().as_millis()
        );

        let order_no_started = Instant::now();
        let data = fetch_order_no(&jk, &wtk).await?;
        tracing::info!(
            "[JK Pay] getStoreAndOrderNo attempt={} elapsed_ms={}",
            attempt + 1,
            order_no_started.elapsed().as_millis()
        );

        let stat_code = data
            .pointer("/stat/code")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        if stat_code < 0 {
            clear_cached_token(redis).await;
            tracing::warn!(
                "[JK Token] stat.code={} on attempt {}, clearing cache and retrying",
                stat_code,
                attempt
            );
            if attempt == 0 {
                continue;
            }
            return Err(format!("获取订单号失败(code={}): {}", stat_code, data));
        }

        let order_no = extract_order_no(&data)?;
        tracing::info!("[JK Pay] orderNo={}", order_no);

        return match try_pay_with_password(
            &jk,
            &wtk,
            card_no,
            card_password,
            &order_no,
            amount_yuan,
            &line,
        )
        .await
        {
            Ok(r) => {
                tracing::info!(
                    "[JK Pay] total elapsed_ms={} attempt={}",
                    started.elapsed().as_millis(),
                    attempt + 1
                );
                Ok(r)
            }
            Err(e) => {
                if should_clear_cached_token(&e) {
                    clear_cached_token(redis).await;
                    tracing::warn!(
                        "[JK Token] cleared cached token due to auth-like error: {}",
                        e
                    );
                } else {
                    tracing::warn!("[JK Pay] non-token error, keeping cached token: {}", e);
                }
                Err(e)
            }
        };
    }

    Err("获取订单号重试失败".to_string())
}

pub(super) async fn query_health_card_balance<C>(
    redis: &mut C,
    seller_username: &str,
    seller_password: &str,
    card_no: &str,
    card_password: &str,
) -> Result<f64, String>
where
    C: ConnectionLike + Send,
{
    let jk = JkClient::new()?;
    let mut last_err: Option<String> = None;

    for attempt in 0..2u32 {
        let wtk = get_token(redis, &jk, seller_username, seller_password).await?;
        match query_payment_channel(&jk, &wtk, card_no, card_password).await {
            Ok((_enc_pwd, balance)) => {
                return balance.ok_or_else(|| "第三方未返回健康卡余额".to_string());
            }
            Err(err) => {
                last_err = Some(err.clone());
                if attempt == 0 {
                    tracing::warn!(
                        "[JK Balance] query failed, clearing cached token and retrying: {}",
                        err
                    );
                    clear_cached_token(redis).await;
                    continue;
                }

                if !card_password.trim().is_empty() {
                    tracing::warn!(
                        "[JK Balance] primary query failed, retrying without payment password: {}",
                        err
                    );
                    let wtk = get_token(redis, &jk, seller_username, seller_password).await?;
                    let (_enc_pwd, balance) = query_payment_channel(&jk, &wtk, card_no, "").await?;
                    return balance.ok_or_else(|| "第三方未返回健康卡余额".to_string());
                }

                return Err(err);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| "第三方未返回健康卡余额".to_string()))
}
