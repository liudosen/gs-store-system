export const veteranProjects = [
  {
    id: "escort-medical",
    title: "陪诊与就医协助",
    note: "陪诊、取号、院内协助与就医流程陪同。",
  },
  {
    id: "home-companion",
    title: "长者陪伴与到家看护",
    note: "面向长者家庭的日常陪伴、照看与到家支持。",
  },
  {
    id: "home-cleaning",
    title: "家政与轻保洁",
    note: "轻家务、居家整理与服务前后基础保洁。",
  },
  {
    id: "meal-delivery",
    title: "送餐与生活代办",
    note: "送餐到家、生活代办与日常外出协助。",
  },
  {
    id: "community-support",
    title: "社区活动支持",
    note: "社区活动执行、现场支持与长者陪同服务。",
  },
];

export const trainingModules = [
  {
    id: "identity-review",
    title: "身份审核与平台规则",
    duration: "35 分钟",
    status: "已完成",
    hint: "确认身份资料、平台规则与接单边界。",
  },
  {
    id: "service-etiquette",
    title: "养老服务礼仪规范",
    duration: "48 分钟",
    status: "进行中",
    hint: "完成服务礼仪、沟通方式与陪护规范。",
  },
  {
    id: "safety-collaboration",
    title: "陪诊与安全协同训练",
    duration: "62 分钟",
    status: "待开始",
    hint: "熟悉安全协同、异常上报与服务记录。",
  },
  {
    id: "readiness-assessment",
    title: "上岗实训与考核说明",
    duration: "40 分钟",
    status: "待开始",
    hint: "了解考核标准、试运行流程与排班要求。",
  },
];

export const veteranProjectTitleMap = Object.fromEntries(
  veteranProjects.map((project) => [project.id, project.title]),
);
