use minio::s3::{
    client::Client,
    http::BaseUrl,
};

#[derive(Clone, Debug)]
enum Platform {
    Invalid,
    Aws,
    Oracle,
    Tencent,
    Baidu,
}

impl From<&str> for Platform {
    fn from(platform_str: &str) -> Self {
        match platform_str.to_lowercase().as_str() {
            "aws" => Platform::Aws,
            "oracle" => Platform::Oracle,
            "tencent" => Platform::Tencent,
            "baidu" => Platform::Baidu,
            _ => Platform::Invalid,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidPlatform(String),
    Other(crate::Error),
}

impl From<&str> for Error {
    fn from(src: &str) -> Self {
        Error::Other(src.into())
    }
}

impl From<String> for Error {
    fn from(src: String) -> Self {
        Error::Other(src.to_string().into())
    }
}


struct Config {
    /// let client read and write with the fixed prefix.
    pub prefix: String,
    pub platform: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    // only work in tencent
    pub app_id: String,
    // only work in oracle
    pub namespace: String,
}

impl Config {
    fn endpoint(&self) -> Result<String, Error> {
        match Platform::from(self.platform.as_str()) {
            Platform::Aws => Ok(String::from("s3.amazonaws.com")),
            Platform::Baidu => Ok(format!("s3.{}.bcebos.com", &self.region)),
            Platform::Tencent => Ok(format!("cos.{}.myqcloud.com", &self.region)),
            Platform::Oracle => {
                if &self.namespace == "" || &self.region == "" {
                    Err(format!("invalid region:{} or namespace:{}", &self.region, &self.namespace).into())
                } else {
                    Ok(format!("{}.compat.objectstorage.{}.oraclecloud.com", &self.namespace, &self.region))
                }
            }
            Platform::Invalid => Err(format!("unknown platform: {}", self.platform.as_str()).into()),
        }
    }
}


struct AwsKV<'minio_client> {
    client: Client<'minio_client>,
    config: Config,
}

impl<'minio_client> AwsKV<'_> {
    pub fn new(c: Config) -> Result<AwsKV<'minio_client>, Error> {
        let base_url = BaseUrl::from_string(c.endpoint()?)?;
        let mut client = Client::new(

        );
        Ok(AwsKV {
            client: Client::new(
                base_url.
            ),
            config: c,
        })
    }
}

