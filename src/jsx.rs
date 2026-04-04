use rquickjs::{Ctx, Result};

pub fn setup_jsx_runtime<'js>(ctx: &Ctx<'js>) -> Result<()> {
    let html = r#"
function note(tag, props, ...children) {
    if (tag === note) {
        return children.flat().join('');
    }
    if (typeof tag === 'function') {
        const p = Object.assign({}, props || {});
        if (children.length === 1) {
            p.children = children[0];
        } else if (children.length > 1) {
            p.children = children;
        }
        return tag(p);
    }
    let attrs = '';
    if (props) {
        for (const key of Object.keys(props)) {
            if (key === 'children') continue;
            const val = props[key];
            if (val === true) {
                attrs += ' ' + key;
            } else if (val !== false && val != null) {
                attrs += ' ' + key + '="' + String(val).replace(/&/g,'&amp;').replace(/"/g,'&quot;') + '"';
            }
        }
    }
    const voidTags = new Set(['area','base','br','col','embed','hr','img','input','link','meta','param','source','track','wbr']);
    const kids = (children || []).flat().join('');
    if (voidTags.has(tag)) {
        return '<' + tag + attrs + '>';
    }
    return '<' + tag + attrs + '>' + kids + '</' + tag + '>';
}
"#;
    ctx.eval::<(), _>(html)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rquickjs::Context;

    #[test]
    fn test_setup_jsx_runtime() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let note: rquickjs::Function = ctx.globals().get("note").unwrap();
            assert!(note.is_function());
        });
    }

    #[test]
    fn test_note_simple_element() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let result: String = ctx.eval(r#"note("div", null, "Hello")"#).unwrap();
            assert_eq!(result, "<div>Hello</div>");
        });
    }

    #[test]
    fn test_note_with_attributes() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let result: String = ctx
                .eval(r#"note("a", { href: "https://example.com" }, "Link")"#)
                .unwrap();
            assert_eq!(result, r#"<a href="https://example.com">Link</a>"#);
        });
    }

    #[test]
    fn test_note_self_closing() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let result: String = ctx.eval(r#"note("br", null)"#).unwrap();
            assert_eq!(result, "<br>");
        });
    }

    #[test]
    fn test_note_nested() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let result: String = ctx
                .eval(r#"note("div", null, note("span", null, "inner"))"#)
                .unwrap();
            assert_eq!(result, "<div><span>inner</span></div>");
        });
    }

    #[test]
    fn test_note_fragment() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let result: String = ctx
                .eval(r#"note(note, null, note("div", null, "A"), note("div", null, "B"))"#)
                .unwrap();
            assert_eq!(result, "<div>A</div><div>B</div>");
        });
    }

    #[test]
    fn test_note_function_component() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            ctx.eval::<(), _>(
                r#"
                function Greeting(props) {
                    return note("span", null, "Hello, " + props.name);
                }
                "#,
            )
            .unwrap();
            let result: String = ctx.eval(r#"note(Greeting, { name: "World" })"#).unwrap();
            assert_eq!(result, "<span>Hello, World</span>");
        });
    }

    #[test]
    fn test_note_boolean_attr_true() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let result: String = ctx.eval(r#"note("input", { disabled: true })"#).unwrap();
            assert_eq!(result, "<input disabled>");
        });
    }

    #[test]
    fn test_note_boolean_attr_false() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let result: String = ctx.eval(r#"note("input", { disabled: false })"#).unwrap();
            assert_eq!(result, "<input>");
        });
    }

    #[test]
    fn test_note_escapes_attr_values() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let result: String = ctx
                .eval(r#"note("div", { title: "a & b" }, "test")"#)
                .unwrap();
            assert_eq!(result, r#"<div title="a &amp; b">test</div>"#);
        });
    }
}
