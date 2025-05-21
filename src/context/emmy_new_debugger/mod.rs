use serde::{Deserialize, Serialize};

/// accpet number as integer
pub mod port_deserializer {
    use serde::{de, Deserializer};
    use std::fmt;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u16, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PortVisitor;

        impl<'de> de::Visitor<'de> for PortVisitor {
            type Value = u16;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("一个整数或能转换为整数的小数")
            }

            // 处理整数
            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value > u16::MAX as u64 {
                    return Err(E::custom(format!("端口号超出范围: {}", value)));
                }
                Ok(value as u16)
            }

            // 处理有符号整数
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value < 0 || value > u16::MAX as i64 {
                    return Err(E::custom(format!("端口号超出范围: {}", value)));
                }
                Ok(value as u16)
            }

            // 处理浮点数
            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value < 0.0 || value > u16::MAX as f64 {
                    return Err(E::custom(format!("端口号超出范围: {}", value)));
                }
                Ok(value as u16)
            }
        }

        deserializer.deserialize_any(PortVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmmyNewDebugArguments {
    pub host: String,
    #[serde(deserialize_with = "port_deserializer::deserialize")]
    pub port: u16,
    pub ext: Vec<String>,
    pub ide_connect_debugger: bool,
    pub source_paths: Vec<String>,
}
