//! `shadow-rs` is a build script write by Rust
//!
//! It's can record compiled project much information.
//! Like version info,dependence info.Like shadow,if compiled,never change.forever follow your project.
//!
//! Generated rust const by exec:`cargo build`
//!
//! # Example
//!
//! ```rust
//! pub const RUST_VERSION :&str = "rustc 1.45.0 (5c1f21c3b 2020-07-13)";
//! pub const BUILD_RUST_CHANNEL :&str = "debug";
//! pub const COMMIT_AUTHOR :&str = "baoyachi";
//! pub const BUILD_TIME :&str = "2020-08-16 13:48:52";
//! pub const COMMIT_DATE :&str = "2020-08-16 13:12:52";
//! pub const COMMIT_EMAIL :&str = "xxx@gmail.com";
//! pub const PROJECT_NAME :&str = "shadow-rs";
//! pub const RUST_CHANNEL :&str = "stable-x86_64-apple-darwin (default)";
//! pub const BRANCH :&str = "master";
//! pub const CARGO_LOCK :&str = r#"
//! ├── chrono v0.4.19
//! │   ├── libc v0.2.80
//! │   ├── num-integer v0.1.44
//! │   │   └── num-traits v0.2.14
//! │   │       [build-dependencies]
//! │   │       └── autocfg v1.0.1
//! │   ├── num-traits v0.2.14 (*)
//! │   └── time v0.1.44
//! │       └── libc v0.2.80
//! └── git2 v0.13.12
//! ├── log v0.4.11
//! │   └── cfg-if v0.1.10
//! └── url v2.2.0
//! ├── form_urlencoded v1.0.0
//! │   └── percent-encoding v2.1.0
//! └── percent-encoding v2.1.0"#;
//! pub const CARGO_VERSION :&str = "cargo 1.45.0 (744bd1fbb 2020-06-15)";
//! pub const BUILD_OS :&str = "macos-x86_64";
//! pub const COMMIT_HASH :&str = "386741540d73c194a3028b96b92fdeb53ca2788a";
//! pub const PKG_VERSION :&str = "0.3.13";
//! ```
//! # Quick Start
//!
//! ## step 1
//! in your `cargo.toml` `packgae` with package add with below config
//!
//! ```toml
//! [package]
//! build = "build.rs"
//!
//! [build-dependencies]
//! shadow-rs = "0.3"
//! ```
//!
//! ## step 2
//! in your project add file `build.rs`,then add with below config
//!
//! ```rust
//! fn main() -> shadow_rs::SdResult<()> {
//!    let src_path = std::env::var("CARGO_MANIFEST_DIR")?;
//!    let out_path = std::env::var("OUT_DIR")?;
//!    shadow_rs::Shadow::build(src_path, out_path)?;
//!    Ok(())
//! }
//! ```
//!
//! ## step 3
//! in your project find `bin` rust file.
//! It's usually `main.rs`, you can find `[bin]` file with `Cargo.toml`,then add with below config
//! ```rust
//! pub mod shadow{
//!    include!(concat!(env!("OUT_DIR"), "/shadow.rs"));
//! }
//! ```
//!
//! ## step 4
//! then you can use const that's shadow build it.
//! ```rust
//! fn main() {
//!    println!("{}",shadow::BRANCH); //master
//!    println!("{}",shadow::SHORT_COMMIT);//8405e28e
//!    println!("{}",shadow::COMMIT_HASH);//8405e28e64080a09525a6cf1b07c22fcaf71a5c5
//!    println!("{}",shadow::COMMIT_DATE);//2020-08-16T06:22:24+00:00
//!    println!("{}",shadow::COMMIT_AUTHOR);//baoyachi
//!    println!("{}",shadow::COMMIT_EMAIL);//xxx@gmail.com
//!
//!    println!("{}",shadow::BUILD_OS);//macos-x86_64
//!    println!("{}",shadow::RUST_VERSION);//rustc 1.45.0 (5c1f21c3b 2020-07-13)
//!    println!("{}",shadow::RUST_CHANNEL);//stable-x86_64-apple-darwin (default)
//!    println!("{}",shadow::CARGO_VERSION);//cargo 1.45.0 (744bd1fbb 2020-06-15)
//!    println!("{}",shadow::PKG_VERSION);//0.3.13
//!    println!("{}",shadow::CARGO_TREE); //like command:cargo tree
//!
//!    println!("{}",shadow::PROJECT_NAME);//shadow-rs
//!    println!("{}",shadow::BUILD_TIME);//2020-08-16 14:50:25
//!    println!("{}",shadow::BUILD_RUST_CHANNEL);//debug
//! }
//!```
//!
//! ## Clap example
//! And you can also use const with [clap](https://github.com/baoyachi/shadow-rs/blob/master/example_shadow/src/main.rs#L24_L26).
//!

mod build;
pub mod channel;
mod ci;
mod env;
pub mod err;
mod git;

use build::*;
use env::*;

use git::*;

use crate::ci::CIType;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env as std_env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use chrono::Local;
pub use err::{SdResult, ShadowError};

const SHADOW_RS: &str = "shadow.rs";

#[derive(Debug)]
pub struct Shadow {
    f: File,
    map: HashMap<ShadowConst, RefCell<ConstVal>>,
    std_env: HashMap<String, String>,
}

impl Shadow {
    fn get_env() -> HashMap<String, String> {
        let mut env_map = HashMap::new();
        for (k, v) in std_env::vars() {
            env_map.insert(k, v);
        }
        env_map
    }

    /// try get current ci env
    fn try_ci(&self) -> CIType {
        if let Some(c) = self.std_env.get("GITLAB_CI") {
            if c == "true" {
                return CIType::Gitlab;
            }
        }

        if let Some(c) = self.std_env.get("GITHUB_ACTIONS") {
            if c == "true" {
                return CIType::Github;
            }
        }

        //TODO completed [travis,jenkins] env

        CIType::None
    }

    pub fn build(src_path: String, out_path: String) -> SdResult<()> {
        let out = {
            let path = Path::new(out_path.as_str());
            if !out_path.ends_with('/') {
                path.join(format!("{}/{}", out_path, SHADOW_RS))
            } else {
                path.join(SHADOW_RS)
            }
        };

        let mut shadow = Shadow {
            f: File::create(out)?,
            map: Default::default(),
            std_env: Default::default(),
        };
        shadow.std_env = Self::get_env();

        let ci_type = shadow.try_ci();
        let src_path = Path::new(src_path.as_str());

        let mut map = new_git(&src_path, ci_type, &shadow.std_env);
        for (k, v) in new_project(&shadow.std_env) {
            map.insert(k, v);
        }
        for (k, v) in new_system_env(&shadow.std_env) {
            map.insert(k, v);
        }
        shadow.map = map;

        shadow.gen_const()?;
        println!("shadow build success");
        Ok(())
    }

    fn gen_const(&mut self) -> SdResult<()> {
        self.write_header()?;
        for (k, v) in self.map.clone() {
            self.write_const(k, v)?;
        }
        Ok(())
    }

    fn write_header(&self) -> SdResult<()> {
        let desc = format!(
            r#"/// Code generated by shadow-rs generator. DO NOT EDIT.
/// Author by https://www.github.com/baoyachi
/// The build script repository:https://github.com/baoyachi/shadow-rs
/// create time by:{}"#,
            Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
        );
        writeln!(&self.f, "{}\n\n", desc)?;
        Ok(())
    }

    fn write_const(&mut self, shadow_const: ShadowConst, val: RefCell<ConstVal>) -> SdResult<()> {
        let val = val.into_inner();
        let desc = format!("/// {}", val.desc);

        let (t, v) = match val.t {
            ConstType::OptStr => (ConstType::Str.to_string(), "".into()),
            ConstType::Str => (ConstType::Str.to_string(), val.v),
        };

        let define = format!(
            "pub const {} :{} = r#\"{}\"#;",
            shadow_const.to_ascii_uppercase(),
            t,
            v
        );
        writeln!(&self.f, "{}", desc)?;
        writeln!(&self.f, "{}\n", define)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() -> SdResult<()> {
        Shadow::build("./".into(), "./".into())?;
        Ok(())
    }
}
