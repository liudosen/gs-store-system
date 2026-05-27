# DESIGN.md

## Scope

This document is the single source of truth for the `侠到家` H5 app UI refactor.

Applies to:

- `frontend/src/pages/CustomerLoginPage.vue`
- `frontend/src/pages/CustomerHomePage.vue`
- `frontend/src/pages/CustomerServicesPage.vue`
- `frontend/src/pages/CustomerOrdersPage.vue`
- `frontend/src/pages/CustomerAddressesPage.vue`
- `frontend/src/pages/CustomerProfilePage.vue`
- shared H5 app styles, components, and layout tokens

Does not apply to:

- the corporate website landing pages
- the veteran onboarding portal
- backend/admin-only views unless explicitly migrated into the same visual system

---

## Product Positioning

`侠到家` is a service-oriented H5 app for families. Its current MVP scenario is in-home support and related doorstep services, but the product tone must reflect the broader `mcx` platform identity:

- trustworthy
- disciplined
- stable
- warm but not soft
- efficient without looking cold

This is not:

- a marketing landing page
- a SaaS dashboard
- a luxury lifestyle app
- a playful youth app
- a generic "AI-generated modern UI"

The visual impression should feel like:

- a Chinese service app
- high-trust
- mobile-first
- practical
- polished enough to ship

---

## Core Principles

Every future UI decision should pass these rules:

1. Mobile app first, not website first.
2. Chinese reading rhythm first, not English typography rhythm first.
3. Service completion matters more than visual spectacle.
4. Information hierarchy must be obvious within 2 seconds.
5. One screen should do one main job.
6. Users should feel "clear and steady", not "wow this is flashy".
7. Visual consistency is more important than per-screen novelty.

---

## Experience Keywords

Use these as the design-language baseline:

- 可信
- 稳定
- 清晰
- 克制
- 轻量
- 可靠
- 有秩序

Avoid these directions:

- 炫技
- 网页宣传感
- SaaS 仪表盘感
- 潮流实验感
- 少女/轻奢/梦幻感
- 过强的科技蓝紫渐变感

---

## App Form Rules

The H5 app must look and behave like an app shell, not a responsive website.

Required shell patterns:

- top header with compact title area
- bottom tab bar with 4 primary destinations
- card/list/sheet based content structure
- strong vertical flow
- clear primary CTA per screen
- content width optimized for phone screens first

Forbidden shell patterns:

- desktop sidebar
- hero banner like a website
- marketing-first above-the-fold layout
- oversized empty white space
- giant headline with low information density
- mixed desktop/mobile interaction patterns on the same screen

---

## Visual Direction

### Overall Style

The app should feel like a blend of:

- Chinese local-service app structure
- calm healthcare/service trust cues
- military-adjacent discipline in spacing and hierarchy

Not literal military styling.
No camouflage.
No patriotic visual cliché.
No badge-heavy fake authority design.

### Visual Mood

- bright warm background
- calm dark blue as control color
- muted green as secondary support
- warm neutral surfaces
- restrained accent for alerts and emphasis

---

## Color System

Use semantic tokens. Do not hardcode random new colors in feature pages.

### Primary Palette

- `--xjd-bg-page`: warm neutral background
- `--xjd-bg-surface`: primary card surface
- `--xjd-bg-surface-soft`: soft grouped surface
- `--xjd-brand`: trustworthy deep blue
- `--xjd-brand-strong`: darker blue for primary CTA active state
- `--xjd-support`: muted service green
- `--xjd-accent`: restrained warm alert red
- `--xjd-text-main`: strong readable dark text
- `--xjd-text-muted`: secondary description text
- `--xjd-line`: low-contrast divider/border

### Color Rules

- primary actions use `--xjd-brand`
- informative support areas may use tinted blue or muted green
- warning/error states use restrained red, never neon red
- avoid more than 1 major accent on a screen
- do not add purple, pink, or bright cyan accents
- do not use gradients unless they are very subtle and localized

### Contrast Rules

- all main text must pass readable contrast on warm backgrounds
- muted text should still be comfortably readable on mobile
- disabled states must remain legible, not washed out

---

## Typography

### Tone

Typography should feel Chinese-first, service-first, and practical.

Not suitable:

- exaggerated display typography
- decorative serif headings
- trendy editorial typography
- excessive letter-spacing

### Hierarchy

Allowed hierarchy:

- App name / key headline
- Screen title
- Section title
- Card title
- Body text
- Secondary helper text
- Label / meta text

### Typography Rules

- screen titles should be compact, not giant
- section titles should communicate function, not marketing copy
- body text should stay short and useful
- helper text should explain next action or status
- avoid long paragraphs inside the app
- 2 lines of explanation is usually the upper bound

### Chinese Content Rules

- prioritize concise Chinese labels
- avoid translated-English product wording
- avoid mixed Chinese/English unless there is a clear product need
- use Chinese punctuation consistently

---

## Layout System

### Spacing Rhythm

Use a consistent mobile spacing rhythm.

Recommended scale:

- 4
- 8
- 12
- 16
- 20
- 24
- 32

Do not use arbitrary spacing unless there is a strong reason.

### Corner Radius

The app should use a unified soft radius system.

- small interactive elements: medium radius
- chips/pills: full radius
- cards/sheets: larger radius
- do not mix sharp corners with large rounded cards on the same screen

### Elevation

Use very restrained depth:

- layer 0: page background
- layer 1: standard cards/lists
- layer 2: emphasized cards or sticky app elements
- layer 3: modal/sheet only

Do not use shadow-heavy "AI card grid" styling.

### Density

This app should be medium density:

- denser than a marketing page
- lighter than an enterprise dashboard

Goal:

- enough information to feel useful
- enough spacing to feel trustworthy

---

## Component Rules

### Buttons

Primary button:

- one per major screen section at most
- visually strong
- full-width when used in forms

Secondary button:

- outline or soft-tint style
- used for fallback actions only

Text button:

- small supporting navigation/action only

Do not:

- place 3 visually equal CTAs together
- use multiple primary buttons in one card
- rely on tiny text links for important actions

### Cards

Cards should feel like mobile functional blocks, not landing-page marketing tiles.

Card rules:

- one clear purpose per card
- compact heading
- short supporting text
- action grouped clearly
- no decorative filler blocks

Avoid:

- giant empty cards
- equal-height grid cards for everything
- too many visual styles on one page

### Chips

Used for:

- region selection
- lightweight filtering
- state grouping

Rules:

- compact
- easy thumb tapping
- active state obvious by fill and text contrast
- no excessive color variety

### List Rows

For service items, addresses, orders, and profile entries:

- prefer row-based mobile layout over desktop card grids
- key info should appear in scanning order
- status/badge aligned consistently
- secondary meta line below, not scattered

### Forms

Forms should feel like service booking forms, not admin forms.

Rules:

- visible labels always
- short helper text when needed
- one column only
- error appears near field or submit zone
- submit area always obvious

Avoid:

- long intimidating forms
- multi-column layouts on mobile
- placeholder-only inputs

### Tab Bar

The bottom tab bar is a core product anchor.

Rules:

- 4 tabs only for now
- labels short and Chinese
- active state highly obvious
- fixed safe-area spacing
- should feel stable across all H5 app pages

Do not:

- restyle tab bar differently per page
- add large icons unless a unified icon system is introduced
- turn the tab bar into a floating decorative object with too much visual noise

---

## Screen Templates

### 1. Login Screen

Purpose:

- reassure
- explain value quickly
- drive SMS login

Required structure:

- compact brand area
- trust/value explanation
- one login card
- phone + verification code flow
- one strong login CTA

Do not:

- make it look like a landing page hero
- over-explain brand story
- use huge imagery with weak form clarity

### 2. Home Screen

Purpose:

- orient user
- expose core tasks
- surface latest order/service context

Required structure:

- compact top context
- one primary summary card
- quick actions
- service preview
- latest order/status module

Do not:

- overload with too many sections
- show generic promotional copy
- make the first screen look like corporate website content

### 3. Services Screen

Purpose:

- browse service items
- select region
- create booking

Required structure:

- region selector
- service list in app-style rows or compact cards
- selected service state obvious
- booking form always visible or clearly attached

Do not:

- use desktop-style product grids
- bury booking action too deep
- create visual disconnect between service list and booking form

### 4. Orders Screen

Purpose:

- track progress
- inspect details

Required structure:

- order list
- selected order detail
- status clarity first
- timeline/progress language if available

Do not:

- make it look like a CRM table
- hide current status
- scatter important service details across too many sections

### 5. Addresses Screen

Purpose:

- manage common addresses quickly

Required structure:

- address list first
- create/edit form second
- default address state obvious

Do not:

- overload with management jargon
- use tiny tap areas for edit behaviors

### 6. Profile Screen

Purpose:

- reassure identity
- expose account and service-management utilities

Required structure:

- profile summary
- common actions
- service explanation / logout area

Do not:

- turn it into a settings maze
- overload with low-priority controls

---

## Copywriting Rules

The app copy should sound:

- clear
- direct
- calm
- service-oriented

The app copy should not sound:

- corporate slogan-heavy
- overly emotional
- internet slang heavy
- translated from English

### Good Copy Patterns

- action-first
- short verbs
- explicit result
- explain what happens next

Examples of preferred style:

- 立即预约
- 查看订单进度
- 选择服务地区
- 添加常用地址
- 提交后系统会进入派单流程

Avoid style like:

- 开启您的高品质服务体验
- 解锁便捷生活新方式
- 我们将为您赋能家庭服务

---

## Motion Rules

Motion should communicate state change, not decoration.

Allowed:

- subtle press feedback
- slight card hover/lift where appropriate on H5
- smooth active-state transitions
- short sheet/modal transitions

Duration guideline:

- 120ms to 220ms for small interactions

Avoid:

- large animated entrances
- floating/glowing effects
- parallax
- decorative continuous motion

---

## App-Like Interaction Rules

To keep the H5 app feeling like an app:

- prioritize thumb reach
- keep major actions in lower or center zones when possible
- use list flows instead of desktop-style page jumping
- maintain safe bottom padding above the tab bar
- keep headers compact and repeatable

If a screen feels like a webpage, fix by:

- reducing headline scale
- increasing information density
- replacing large promo cards with functional cards
- converting equal grids into rows/lists
- reinforcing the fixed app shell

---

## Anti-Patterns

These are banned unless explicitly approved:

- purple/blue AI gradient hero sections
- giant marketing hero banners inside app screens
- fake dashboard widgets
- too many shadow layers
- oversized rounded cards everywhere with no hierarchy
- mixed visual languages per page
- empty aesthetic-first space that reduces task clarity
- website navbar patterns inside the H5 app
- random English labels in core app navigation
- flashy illustration-first login screens

---

## Refactor Order

All future H5 refactors should follow this sequence:

1. stabilize shell
2. stabilize tokens
3. rebuild login
4. rebuild home
5. rebuild services
6. rebuild orders
7. rebuild addresses
8. rebuild profile
9. unify copy
10. polish motion and states

Never redesign each page independently without first checking this document.

---

## Implementation Rules For Codex / LLM

When generating or refactoring UI for this H5 app:

- always treat it as a mobile service app
- always prefer Chinese UI copy
- always reuse existing semantic tokens before introducing new ones
- always preserve bottom-tab navigation consistency
- always optimize for real task flow over visual novelty

Before writing code, check:

- does this look like an app or a website
- does this screen have one clear primary task
- is the hierarchy understandable in one glance
- does the copy sound like a Chinese service product
- is the card/list density appropriate for mobile

If the answer to any of these is no, revise before implementation.

---

## Definition of Success

The redesign is successful when a user opening `侠到家` feels:

- this is a real Chinese service app
- I know what to do next
- the product feels trustworthy
- booking and order tracking feel easy
- the interface is cleaner and more professional than a generic H5 template

The redesign is not successful if the result still feels like:

- a company website squeezed into mobile
- a generic AI-generated Tailwind demo
- a desktop dashboard translated to phone

