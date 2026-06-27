# 股票行情看板 - 技术设计文档

## 1. 项目概述

**目标**：构建一个基于 Rust 的服务端渲染网站，采集股票关联的大宗商品行情数据，聚合存储并分析价格趋势，辅助投资决策。

**设计原则**：
- 不使用前后端分离：服务端渲染为主，htmx 做局部交互增强
- 低资源消耗：面向 2C2G 云服务器，Rust 全栈
- 渐进式开发：先跑通数据采集到存储到展示，再逐步丰富功能
- 关注点分离：采集、存储、分析、展示各层解耦，通过 trait 定义边界

**核心用户场景**：
1. 打开首页，看到持仓股票关联商品的最新价格及趋势
2. 进入商品详情页，查看历史价格走势图
3. 系统每日自动采集，采集后自动分析趋势，在页面上标识上涨/下跌

## 2. 技术选型

| 项目 | 选型 | 版本 | 引入方式 |
|---|---|---|---|
| 语言 | Rust | stable（2024 edition） | - |
| 异步运行时 | Tokio | 1.x | Cargo |
| Web 框架 | Axum | 0.8.x | Cargo |
| 模板引擎 | Tera | 1.x | Cargo |
| HTTP 客户端 | reqwest | 0.12.x | Cargo |
| HTML 解析 | scraper | 0.22.x | Cargo |
| 序列化 | serde + serde_json | 1.x | Cargo |
| 定时调度 | tokio::time | 内置于 Tokio | - |
| 样式框架 | Bootstrap | 5.x | CDN |
| 前端增强 | htmx | 2.x | CDN |
| 图表 | Chart.js | 4.x | CDN |
| 数据存储 | JSON 文件 | - | 本地文件系统 |

**选型理由**：
- **Axum**：Tokio 团队出品，与 reqwest/tokio::fs/tokio::time 零摩擦协作，API 简洁适合学习
- **Tera**：运行时模板引擎，修改模板无需重编译，开发迭代快
- **scraper**：基于 CSS 选择器的 HTML 解析，适合结构固定的采集
- **Bootstrap 5**：组件齐全，CDN 一行引入，无构建工具依赖
- **htmx**：HTML 属性声明异步交互，与原生 JS 互不冲突
- **Chart.js**：轻量图表库，CDN 引入，适合价格趋势折线图

## 3. 系统架构

```
浏览器: Bootstrap 5 + htmx + Chart.js + 原生 JS
        | HTTP
Axum Server: Handlers(页面) + API(htmx) + Static Files
        |
    Tera 模板渲染
        |
分析引擎(趋势判断/均线计算) + 存储层(JSON 文件读写)
        |
采集层(Scraper trait): ZhuJiaScraper(猪价网) / PPIScraper(生意社)
        |
reqwest + scraper → 外部数据源
```

## 4. 目录结构

```
stock/
├── Cargo.toml
├── docs/TECHNICAL_DESIGN.md
├── src/
│   ├── main.rs                 # 入口：运行时、路由、定时任务
│   ├── config.rs               # 采集目标配置
│   ├── models/
│   │   ├── mod.rs
│   │   └── commodity.rs        # 商品价格模型
│   ├── scrapers/
│   │   ├── mod.rs              # Scraper trait + 工厂
│   │   ├── zhujia.rs           # 猪价网采集器
│   │   └── ppi.rs             # 生意社采集器
│   ├── storage/
│   │   ├── mod.rs              # Storage trait
│   │   └── json_store.rs       # JSON 文件存储
│   ├── analysis/
│   │   ├── mod.rs
│   │   └── trend.rs            # 趋势分析
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── dashboard.rs        # 首页看板
│   │   ├── commodity_detail.rs # 商品详情页
│   │   └── api.rs              # htmx 局部更新接口
│   └── scheduler.rs            # 定时采集调度
├── templates/
│   ├── base.html               # 布局骨架
│   ├── dashboard.html          # 首页看板
│   ├── commodity_detail.html   # 商品详情页
│   └── partials/               # 局部模板片段
│       ├── price_table.html
│       └── trend_badge.html
├── static/css/custom.css
└── data/commodities/           # 按商品分目录存储 JSON
    ├── live_pig/prices.json
    ├── corn/prices.json
    ├── soybean_meal/prices.json
    ├── titanium_dioxide/prices.json
    └── sulfur/prices.json
```

## 5. 数据模型

### 5.1 CommodityPrice

```rust
struct CommodityPrice {
    date: String,                    // ISO 日期 "2026-06-27"
    name: String,                    // 商品名称
    price: f64,                      // 价格数值
    unit: String,                    // 单位
    change: Option<f64>,             // 较前日涨跌额
    change_percent: Option<f64>,     // 较前日涨跌幅 %
}
```

### 5.2 CommodityConfig

```rust
struct CommodityConfig {
    name: String,
    code: String,        // 内部编码
    source: String,      // 数据源标识 "zhujia" / "ppi"
    url: String,
    unit: String,
}
```

### 5.3 TrendResult

```rust
struct TrendResult {
    commodity_code: String,
    period_days: u32,
    direction: TrendDirection,   // Up / Down / Flat
    consecutive_days: u32,       // 连续涨/跌天数
    change_percent: f64,         // 区间总变化幅度 %
    short_ma: f64,               // 5日均价
    medium_ma: f64,              // 20日均价
}
```

### 5.4 JSON 存储格式

```json
{
  "commodity": { "name": "生猪", "code": "live_pig", "unit": "元/公斤" },
  "prices": [
    { "date": "2026-06-27", "price": 18.5, "change": -0.3, "change_percent": -1.59 }
  ],
  "last_updated": "2026-06-27T10:30:00+08:00"
}
```

## 6. 核心接口设计

### 6.1 Scraper trait

```rust
#[async_trait]
trait Scraper {
    async fn fetch(&self, config: &CommodityConfig) -> Result<Vec<CommodityPrice>>;
    fn source(&self) -> &str;
}
```

### 6.2 Storage trait

```rust
#[async_trait]
trait Storage {
    async fn save(&self, commodity_code: &str, prices: &[CommodityPrice]) -> Result<()>;
    async fn load(&self, commodity_code: &str) -> Result<Vec<CommodityPrice>>;
    async fn load_recent(&self, commodity_code: &str, days: u32) -> Result<Vec<CommodityPrice>>;
}
```

### 6.3 趋势分析

```rust
fn analyze_trend(prices: &[CommodityPrice]) -> TrendResult;
```

判断逻辑：上涨=5日均线>20日均线且近5日涨幅>2%；下跌反之；其余为震荡。

## 7. 路由设计

| 方法 | 路径 | Handler | 说明 |
|---|---|---|---|
| GET | `/` | dashboard | 首页看板，所有商品最新价格+趋势标签 |
| GET | `/commodity/{code}` | commodity_detail | 商品详情页，价格表+趋势图 |
| GET | `/api/refresh` | refresh_all | htmx 触发：重新采集所有数据 |
| GET | `/api/commodity/{code}/prices` | commodity_prices | htmx：价格表格 HTML 片段 |
| GET | `/api/commodity/{code}/chart` | commodity_chart | 图表 JSON 数据（供 Chart.js） |

## 8. 定时采集调度

- 默认每日 11:00 采集（大宗商品价格通常上午出价）
- 启动时若当日未采集，立即执行一次
- 流程：遍历配置 -> 按 source 分组 -> 对应 Scraper 采集 -> 存储 -> 分析 -> 写日志

## 9. 页面设计

**首页看板**：商品名称、最新价格、单位、涨跌幅（绿色↑/红色↓/灰色—）、详情链接，底部显示上次采集时间，"刷新数据"按钮 htmx 触发局部更新。

**商品详情页**：Chart.js 折线图（近30/60/90天可切换）+ 趋势分析文字（连续涨跌天数、均线）+ 历史价格表格。

## 10. 未来预留

- 股票价格采集（models 预留 StockPrice）
- 持仓信息管理（Portfolio 模型，路由预留 `/portfolio`）
- 通知告警（趋势分析触发推送）
- 数据库存储替换 JSON（Storage trait 已抽象）
- 更多数据源（Scraper trait + 配置化）
