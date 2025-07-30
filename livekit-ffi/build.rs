// Copyright 2023 LiveKit, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{env, path::Path};

fn main() {
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    let webrtc_dir = webrtc_sys_build::webrtc_dir();
    println!("cargo:warning=livekit-ffi downloading webrtc to {webrtc_dir:?}");
    webrtc_sys_build::download_webrtc().unwrap();
    if !webrtc_dir.exists() {
        panic!("download_webrtc didn't create webrtc_dir")
    }

    {
        // Copy the webrtc license to CARGO_MANIFEST_DIR
        // (used by the ffi release action)
        let webrtc_dir = webrtc_sys_build::webrtc_dir();
        let license = webrtc_dir.join("LICENSE.md");
        let target_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

        let out_file = Path::new(&target_dir).join("WEBRTC_LICENSE.md");

        std::fs::copy(license, out_file).unwrap();
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    match target_os.as_str() {
        "windows" => {}
        "linux" => {
            println!("cargo:rustc-link-lib=static=webrtc");
        }
        "android" => {
            webrtc_sys_build::configure_jni_symbols().unwrap();
        }
        "macos" | "ios" => {
            println!("cargo:rustc-link-arg=-ObjC");
        }
        _ => {
            panic!("Unsupported target, {}", target_os);
        }
    }
}
