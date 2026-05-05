use rquickjs::{Ctx, Result};

pub fn setup_jsx_runtime<'js>(ctx: &Ctx<'js>) -> Result<()> {
    let html = r#"
function _escapeHtml(str) {
    return String(str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#x27;');
}
function _escapeAttr(str) {
    return String(str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#x27;');
}
function _isHtml(obj) {
    return obj && typeof obj === 'object' && obj.__html !== undefined;
}
function _toHtml(child) {
    if (_isHtml(child)) {
        return child.__html;
    }
    return _escapeHtml(child);
}
function _makeHtml(str) {
    var wrapper = new String(str);
    wrapper.__html = str;
    return wrapper;
}
function note(tag, props, ...children) {
    if (tag === note) {
        return _makeHtml(children.flat(Infinity).map(_toHtml).join(''));
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
                attrs += ' ' + key + '="' + _escapeAttr(val) + '"';
            }
        }
    }
    const voidTags = new Set(['area','base','br','col','embed','hr','img','input','link','meta','param','source','track','wbr']);
    const kids = (children || []).flat(Infinity).map(_toHtml).join('');
    if (voidTags.has(tag)) {
        return _makeHtml('<' + tag + attrs + '>');
    }
    return _makeHtml('<' + tag + attrs + '>' + kids + '</' + tag + '>');
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
            let result: String = ctx.eval(r#"String(note("div", null, "Hello"))"#).unwrap();
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
                .eval(r#"String(note("a", { href: "https://example.com" }, "Link"))"#)
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
            let result: String = ctx.eval(r#"String(note("br", null))"#).unwrap();
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
                .eval(r#"String(note("div", null, note("span", null, "inner")))"#)
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
                .eval(r#"String(note(note, null, note("div", null, "A"), note("div", null, "B")))"#)
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
            let result: String = ctx
                .eval(r#"String(note(Greeting, { name: "World" }))"#)
                .unwrap();
            assert_eq!(result, "<span>Hello, World</span>");
        });
    }

    #[test]
    fn test_note_boolean_attr_true() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let result: String = ctx
                .eval(r#"String(note("input", { disabled: true }))"#)
                .unwrap();
            assert_eq!(result, "<input disabled>");
        });
    }

    #[test]
    fn test_note_boolean_attr_false() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let result: String = ctx
                .eval(r#"String(note("input", { disabled: false }))"#)
                .unwrap();
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
                .eval(r#"String(note("div", { title: "a & b" }, "test"))"#)
                .unwrap();
            assert_eq!(result, r#"<div title="a &amp; b">test</div>"#);
        });
    }

    #[test]
    fn test_note_xss_escapes_children_script_tag() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let result: String = ctx
                .eval(r#"String(note("div", null, "<script>alert(1)</script>"))"#)
                .unwrap();
            assert_eq!(
                result,
                r#"<div>&lt;script&gt;alert(1)&lt;/script&gt;</div>"#
            );
        });
    }

    #[test]
    fn test_note_xss_escapes_children_angle_brackets() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let result: String = ctx
                .eval(r#"String(note("div", null, "<img onerror=alert(1) src=x>"))"#)
                .unwrap();
            assert_eq!(result, r#"<div>&lt;img onerror=alert(1) src=x&gt;</div>"#);
        });
    }

    #[test]
    fn test_note_xss_escapes_attr_with_script_injection() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let result: String = ctx
                .eval(r#"String(note("div", { title: '"><script>alert(1)</script>' }, "test"))"#)
                .unwrap();
            assert_eq!(
                result,
                r#"<div title="&quot;&gt;&lt;script&gt;alert(1)&lt;/script&gt;">test</div>"#
            );
        });
    }

    #[test]
    fn test_note_xss_escapes_attr_single_quote() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let result: String = ctx
                .eval(r#"String(note("div", { onclick: "alert('xss')" }, "test"))"#)
                .unwrap();
            assert_eq!(
                result,
                r#"<div onclick="alert(&#x27;xss&#x27;)">test</div>"#
            );
        });
    }

    #[test]
    fn test_note_xss_escapes_children_in_function_component() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            ctx.eval::<(), _>(
                r#"
                function Safe(props) {
                    return note("div", null, props.children);
                }
                "#,
            )
            .unwrap();
            let result: String = ctx
                .eval(r#"String(note(Safe, null, '<script>alert(1)</script>'))"#)
                .unwrap();
            assert_eq!(
                result,
                r#"<div>&lt;script&gt;alert(1)&lt;/script&gt;</div>"#
            );
        });
    }

    #[test]
    fn test_note_xss_escapes_nested_children() {
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            setup_jsx_runtime(&ctx).unwrap();
            let result: String = ctx
                .eval(r#"String(note("div", null, "safe", note("span", null, "<b>bold</b>"), "end"))"#)
                .unwrap();
            assert_eq!(
                result,
                r#"<div>safe<span>&lt;b&gt;bold&lt;/b&gt;</span>end</div>"#
            );
        });
    }
}
