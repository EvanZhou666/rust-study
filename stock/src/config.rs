use crate::models::commodity::CommodityConfig;

pub fn default_commodities() -> Vec<CommodityConfig> {
    vec![
        CommodityConfig {
            name: "生猪".into(),
            code: "live_pig".into(),
            source: "zhujia".into(),
            url: "https://zhujia.zhuwang.com.cn/indexov.shtml".into(),
            unit: "元/公斤".into(),
        },
        CommodityConfig {
            name: "玉米".into(),
            code: "corn".into(),
            source: "zhujia".into(),
            url: "https://zhujia.zhuwang.com.cn/indexov.shtml".into(),
            unit: "元/吨".into(),
        },
        CommodityConfig {
            name: "豆粕".into(),
            code: "soybean_meal".into(),
            source: "zhujia".into(),
            url: "https://zhujia.zhuwang.com.cn/indexov.shtml".into(),
            unit: "元/吨".into(),
        },
        CommodityConfig {
            name: "钛白粉".into(),
            code: "titanium_dioxide".into(),
            source: "ppi".into(),
            url: "https://m1.100ppi.com/Rawmex/427.html".into(),
            unit: "元/吨".into(),
        },
        CommodityConfig {
            name: "硫磺".into(),
            code: "sulfur".into(),
            source: "ppi".into(),
            url: "https://m1.100ppi.com/Rawmex/645.html".into(),
            unit: "元/吨".into(),
        },
    ]
}