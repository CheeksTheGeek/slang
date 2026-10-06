//! `(* name = value *)` attribute instances via the `getAttributes` mirrors:
//! `Symbol::attributes()`, `Statement::attributes()`, `Expression::attributes()`,
//! and `PortConnection::attributes()`. Ported from slang's own "Attributes"
//! unit test (tests/unittests/ast/MemberTests.cpp). Requested by the sv-sim VPI
//! port for `vpiAttribute` (IEEE 1800-2023 37.83).

use sv_lang::kinds::SymbolKind;
use sv_lang::{Compilation, Session};

const SRC: &str = r#"
module m;
    localparam param = "str val";
    (* foo, bar = 1 *) (* baz = 1 + 2 * 3 *) wire foo, bar;

    (* blah *) n n1((* blah2 *) 0);

    (* blah3 = param *);

    function void func;
    endfunction

    int j;
    always_comb begin : block
        (* blah4 *) func (* blah5 *) ();
        j = 3 + (* blah6 *) 4;
    end
endmodule

module n((* asdf *) input foo);
endmodule
"#;

fn design() -> sv_lang::Design {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(SRC).unwrap();
    comp.compile().unwrap()
}

fn m_body(d: &sv_lang::Design) -> sv_lang::Symbol<'_> {
    d.top_instances()
        .find(|i| i.name() == "m")
        .unwrap()
        .instance_body()
        .unwrap()
}

#[test]
fn symbol_attributes_names_and_values() {
    let d = design();
    let m = m_body(&d);
    // `(* foo, bar = 1 *) (* baz = 1 + 2 * 3 *) wire foo, bar;` -> all three on
    // each declared net. Bare `foo` defaults to 1; `baz` folds 1 + 2*3 = 7.
    let bar = m.find("bar").unwrap();
    let got: Vec<(String, Option<i64>)> = bar
        .attributes()
        .map(|a| {
            (
                a.name().to_string(),
                a.attribute_value().and_then(|c| c.as_i64()),
            )
        })
        .collect();
    assert_eq!(
        got,
        vec![
            ("foo".to_string(), Some(1)),
            ("bar".to_string(), Some(1)),
            ("baz".to_string(), Some(7)),
        ]
    );
}

#[test]
fn instance_and_port_attributes() {
    let d = design();
    let m = m_body(&d);

    let n1 = m.find("n1").unwrap();
    let inst_attrs: Vec<String> = n1.attributes().map(|a| a.name().to_string()).collect();
    assert_eq!(inst_attrs, vec!["blah"]);

    // Port `foo` of n's body carries `(* asdf *)`.
    let foo_port = n1.instance_body().unwrap().find("foo").unwrap();
    let port_attrs: Vec<String> = foo_port
        .attributes()
        .map(|a| a.name().to_string())
        .collect();
    assert_eq!(port_attrs, vec!["asdf"]);
}

#[test]
fn port_connection_attributes() {
    let d = design();
    let m = m_body(&d);
    let n1 = m.find("n1").unwrap();
    let conn = n1.instance_port_connections().next().unwrap();
    let attrs: Vec<String> = conn.attributes().map(|a| a.name().to_string()).collect();
    assert_eq!(attrs, vec!["blah2"]);
}

#[test]
fn statement_and_expression_attributes() {
    let d = design();
    let m = m_body(&d);
    let proc = m
        .members()
        .find(|s| s.kind() == SymbolKind::ProceduralBlock)
        .unwrap();

    // Walk the always_comb body's semantic subtree, collecting attribute names
    // on every statement and expression node.
    let mut found: Vec<String> = Vec::new();
    let mut stack = vec![proc.body().unwrap().as_sem_node()];
    while let Some(node) = stack.pop() {
        if let Some(s) = node.as_statement() {
            found.extend(s.attributes().map(|a| a.name().to_string()));
        }
        if let Some(e) = node.as_expression() {
            found.extend(e.attributes().map(|a| a.name().to_string()));
        }
        stack.extend(node.children());
    }

    // blah4 = the call statement, blah5 = the call expression, blah6 = the
    // `3 + 4` binary expression.
    for want in ["blah4", "blah5", "blah6"] {
        assert!(
            found.contains(&want.to_string()),
            "missing {want}: {found:?}"
        );
    }
}

#[test]
fn empty_member_attribute_carries_a_value() {
    // `(* blah3 = param *);` on a standalone empty member — param is the
    // string literal "str val", which SV stores as a packed bit-vector (an
    // integer ConstantValue, matching slang's own test which must
    // convertToStr). Found by scanning members for the attribute.
    let d = design();
    let m = m_body(&d);
    let holder = m
        .members()
        .find(|s| s.attributes().any(|a| a.name() == "blah3"))
        .expect("a member carrying blah3");
    let attr = holder.attributes().find(|a| a.name() == "blah3").unwrap();
    let value = attr.attribute_value().expect("blah3 folded to a value");
    // "str val" is 7 bytes -> a 56-bit packed integer.
    let svint = value
        .as_integer()
        .expect("string literal stored as an integer");
    assert_eq!(svint.bit_width(), 56);
    // Top byte is 's' (0x73).
    assert_eq!(svint.as_u64().map(|v| (v >> 48) & 0xff), Some(0x73));
}

#[test]
fn no_attributes_is_empty() {
    let d = design();
    let m = m_body(&d);
    // `int j;` has no attributes.
    assert_eq!(m.find("j").unwrap().attributes().count(), 0);
}
