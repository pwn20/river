use std::env;
use std::fs;
use std::path::{PathBuf};

fn main()
{
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let is_win7_target = env::var("CARGO_FEATURE_WIN7").is_ok();

    if target_os == "windows" && is_win7_target
    {
        println!("cargo:warning=>>> [RIVER BUILD SCRIPT] Enabling Windows 7 Compatibility Thunks & DLLs <<<");

        // Inject YY-Thunks hooks into the build pipeline targeting Windows 7
        thunk::thunk();

        println!("cargo:rustc-link-arg=/DELAYLOAD:combase.dll");
        //println!("cargo:rustc-link-arg=/DELAYLOAD:shcore.dll");
        println!("cargo:rustc-link-arg=/INCLUDE:__pfnDliFailureHook2");
        println!("cargo:rustc-link-arg=delayimp.lib");

        copy_win7_dlls();
    }
}

fn copy_win7_dlls()
{
    println!("cargo:warning=>>> [RIVER BUILD SCRIPT] Copying DLLs.. <<<");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let dll_source_dir = manifest_dir.join("mesa3d_dlls");

    // Re-run build script if any DLLs in win7_dlls/ are added or changed
    println!("cargo:rerun-if-changed={}", dll_source_dir.display());

    if !dll_source_dir.exists()
    {
        println!("cargo:warning=>>> [RIVER BUILD SCRIPT] DLL folder not found <<<");
        return;
    }

    // OUT_DIR looks like: target/<target-triple>/<profile>/build/river-<hash>/out
    // Ancestors(3) steps up to: target/<target-triple>/<profile>/
    if let Ok(out_dir) = env::var("OUT_DIR")
    {
        let out_path = PathBuf::from(out_dir);
        if let Some(target_dir) = out_path.ancestors().nth(3)
        {
            if let Ok(entries) = fs::read_dir(&dll_source_dir)
            {
                for entry in entries.flatten()
                {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("dll")
                    {
                        let dest = target_dir.join(path.file_name().unwrap());
                        let _ = fs::copy(&path, &dest);
                    }
                }
            }
        }
    }
}