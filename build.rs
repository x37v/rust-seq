use quote::quote;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    #[cfg(feature = "euclidean")]
    {
        let dest_path = Path::new(&out_dir).join("euclid.rs");
        let mut f = std::fs::File::create(&dest_path)?;

        let mut inits = Vec::new();
        let mut pattern = [false; 64];

        for steps in 2..64 {
            for pulses in 1..steps {
                euclidian_rythms::euclidian_rythm(&mut pattern, pulses, steps);
                let mut v = 0usize;
                for bit in 0..64 {
                    v |= (if pattern[bit] { 1 } else { 0 }) << bit;
                }
                inits.push(quote! {
                    m.insert((#steps, #pulses), #v);
                })
            }
        }

        f.write_all(
            quote! {
            ::lazy_static::lazy_static! {
                static ref EUCLID_STEP_PULSE_PATTERN_MAP: ::std::collections::HashMap<(usize, usize), usize> = {
                    let mut m = ::std::collections::HashMap::new();
                    #(#inits)*
                    m
                };
            }
            }
            .to_string()
            .as_bytes(),
        )?;
    }
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
