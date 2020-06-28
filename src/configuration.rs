use serde_derive::Deserialize;
use packageurl::PackageUrl;
use std::str::FromStr;
use std::error::Error;


#[derive(Debug)]
#[derive(Deserialize)]
pub struct Configuration {
    pub dest: Option<String>,
    pub image: Option<Image>,
    pub dependencies: Vec<Dependency>
}

#[derive(Debug)]
#[derive(Deserialize)]
pub struct Image {
    pub path: String
}

#[derive(Debug)]
#[derive(Deserialize)]
pub struct Dependency {
    pub purl: Option<String>,
    pub provider: Option<String>,
    pub package: Option<String>,
    pub version: Option<String>
}

impl Dependency {
    
    pub fn provider(&self) -> Result<String, Box<dyn Error>> {
        match &self.provider {
            Some(s) => { Ok(s.clone()) }
            None => {
                match PackageUrl::from_str(self.purl.as_ref().unwrap().as_str()) {
                    Ok(pkg) => { Ok(pkg.ty.into_owned()) }
                    Err(e) => { Err(Box::from(e)) }
                }
            }
        }
    }

    pub fn package(&self) -> Result<String, Box<dyn Error>> {
        match &self.package {
            Some(s) => { Ok(s.clone()) }
            None => {
                match PackageUrl::from_str(self.purl.as_ref().unwrap().as_str()) {
                    Ok(pkg) => { Ok(pkg.name.into_owned()) }
                    Err(e) => { Err(Box::from(e)) }
                }
            }
        }
    }

    pub fn version(&self) -> Result<String, Box<dyn Error>> {
        match &self.version {
            Some(s) => { Ok(s.clone()) }
            None => {
                match PackageUrl::from_str(self.purl.as_ref().unwrap().as_str()) {
                    Ok(pkg) => { 
                        match pkg.version {
                            Some(s) => { Ok(s.into_owned()) }
                            None => Ok("".to_string())
                        }
                    }
                    Err(e) => { Err(Box::from(e)) }
                }
            }
        }
    }

}