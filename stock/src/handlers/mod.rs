use std::sync::Arc;
use tera::Tera;
use crate::config::default_commodities;
use crate::models::commodity::CommodityConfig;
use crate::scrapers::ppi::PpiScraper;
use crate::scrapers::zhujia::ZhuJiaScraper;
use crate::storage::json_store::JsonStore;

#[derive(Clone)]
pub struct AppState {
    pub tera: Arc<Tera>,
    pub configs: Arc<Vec<CommodityConfig>>,
    pub storage: Arc<JsonStore>,
    pub zhujia_scraper: Arc<ZhuJiaScraper>,
    pub ppi_scraper: Arc<PpiScraper>,
}

impl AppState {
    pub fn new(data_dir: &str) -> Self {
        Self {
            // tera 类似于jinja的模板引擎
            tera: Arc::new(tera_init()),
            configs: Arc::new(default_commodities()),
            storage: Arc::new(JsonStore::new(data_dir.into())),
            zhujia_scraper: Arc::new(ZhuJiaScraper::new()),
            ppi_scraper: Arc::new(PpiScraper::new()),
        }
    }
}

fn tera_init() -> Tera {
    let mut tera = Tera::new("templates/**/*.html")
        .expect("Failed to initialize Tera templates");
    // 默认在渲染模板内容的时候会执行html特殊符号的转义
    tera.autoescape_on(vec!["html"]);
    tera
}

pub mod dashboard;
pub mod commodity_detail;
pub mod api;
