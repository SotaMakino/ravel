use rquickjs::{ArrayBuffer, Ctx, Error, Result, TypedArray, Value};

/// Extract bytes from a `Uint8Array` or `ArrayBuffer`.
pub fn bytes_from_value(value: &Value<'_>) -> Result<Vec<u8>> {
    if let Ok(array) = TypedArray::<u8>::from_value(value.clone()) {
        if let Some(bytes) = array.as_bytes() {
            return Ok(bytes.to_vec());
        }
    }
    if let Some(object) = value.as_object() {
        if let Some(buffer) = ArrayBuffer::from_object(object.clone()) {
            if let Some(bytes) = buffer.as_bytes() {
                return Ok(bytes.to_vec());
            }
        }
    }
    Err(Error::new_from_js_message(
        "value",
        "bytes",
        "expected a Uint8Array or ArrayBuffer".to_string(),
    ))
}

fn to_uint8_array<'js>(ctx: &Ctx<'js>, bytes: Vec<u8>) -> Result<Value<'js>> {
    let buffer = ArrayBuffer::new(ctx.clone(), bytes)?;
    Ok(TypedArray::<u8>::from_arraybuffer(buffer)?.into_value())
}

pub fn setup_encoding<'js>(ctx: &Ctx<'js>) -> Result<()> {
    ctx.globals().set(
        "__ravel_utf8_encode",
        rquickjs::function::Func::new(|ctx: Ctx<'js>, input: String| -> Result<Value<'js>> {
            to_uint8_array(&ctx, input.into_bytes())
        }),
    )?;

    ctx.globals().set(
        "__ravel_utf8_decode",
        rquickjs::function::Func::new(|input: Value<'js>| -> Result<String> {
            let bytes = bytes_from_value(&input)?;
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        }),
    )?;

    // UTF-8 only. Other labels throw rather than silently mis-decoding.
    ctx.eval::<(), _>(
        r#"
        class TextEncoder {
            get encoding() { return "utf-8"; }
            encode(input) {
                return __ravel_utf8_encode(input === undefined ? "" : String(input));
            }
        }

        class TextDecoder {
            constructor(label) {
                const normalized = String(label === undefined ? "utf-8" : label).toLowerCase();
                if (normalized !== "utf-8" && normalized !== "utf8" && normalized !== "unicode-1-1-utf-8") {
                    throw new RangeError("ravel only supports the utf-8 encoding, got: " + label);
                }
                this._encoding = "utf-8";
            }
            get encoding() { return this._encoding; }
            decode(input) {
                if (input === undefined) return "";
                return __ravel_utf8_decode(input);
            }
        }
        "#,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rquickjs::{Context, Runtime};

    fn with_ctx<F, R>(f: F) -> R
    where
        F: FnOnce(Ctx<'_>) -> R,
    {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_encoding(&ctx).unwrap();
            f(ctx)
        })
    }

    #[test]
    fn test_text_encoder_encodes_ascii() {
        with_ctx(|ctx| {
            let len: usize = ctx
                .eval("new TextEncoder().encode('abc').length")
                .unwrap();
            assert_eq!(len, 3);
        });
    }

    #[test]
    fn test_text_encoder_encodes_multibyte() {
        with_ctx(|ctx| {
            // "日本" is two 3-byte characters in UTF-8.
            let len: usize = ctx.eval("new TextEncoder().encode('日本').length").unwrap();
            assert_eq!(len, 6);
        });
    }

    #[test]
    fn test_text_encoder_encoding_property() {
        with_ctx(|ctx| {
            let encoding: String = ctx.eval("new TextEncoder().encoding").unwrap();
            assert_eq!(encoding, "utf-8");
        });
    }

    #[test]
    fn test_text_encoder_undefined_gives_empty() {
        with_ctx(|ctx| {
            let len: usize = ctx.eval("new TextEncoder().encode().length").unwrap();
            assert_eq!(len, 0);
        });
    }

    #[test]
    fn test_round_trip() {
        with_ctx(|ctx| {
            let out: String = ctx
                .eval("new TextDecoder().decode(new TextEncoder().encode('héllo 世界'))")
                .unwrap();
            assert_eq!(out, "héllo 世界");
        });
    }

    #[test]
    fn test_text_decoder_accepts_array_buffer() {
        with_ctx(|ctx| {
            let out: String = ctx
                .eval("new TextDecoder().decode(new TextEncoder().encode('hi').buffer)")
                .unwrap();
            assert_eq!(out, "hi");
        });
    }

    #[test]
    fn test_text_decoder_undefined_gives_empty() {
        with_ctx(|ctx| {
            let out: String = ctx.eval("new TextDecoder().decode()").unwrap();
            assert_eq!(out, "");
        });
    }

    #[test]
    fn test_text_decoder_rejects_other_encodings() {
        with_ctx(|ctx| {
            let result: rquickjs::Result<String> = ctx.eval("new TextDecoder('latin1')");
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_text_decoder_accepts_utf8_aliases() {
        with_ctx(|ctx| {
            let encoding: String = ctx.eval("new TextDecoder('UTF8').encoding").unwrap();
            assert_eq!(encoding, "utf-8");
        });
    }

    #[test]
    fn test_bytes_from_value_rejects_other_types() {
        with_ctx(|ctx| {
            let value: Value = ctx.eval("({})").unwrap();
            assert!(bytes_from_value(&value).is_err());
        });
    }
}
