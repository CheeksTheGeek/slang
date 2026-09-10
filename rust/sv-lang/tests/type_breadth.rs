//! Type-breadth accessors: enum base/members, struct/union fields with their
//! offsets, and array element types.

use sv_lang::{Compilation, Session};

const SRC: &str = "\
module m;
    typedef enum logic [1:0] { RED, GREEN, BLUE } color_t;
    typedef struct packed { logic [3:0] hi; logic [3:0] lo; } byte_t;
    color_t c;
    byte_t b;
    logic [7:0] mem [4];
endmodule
";

fn compile(src: &str) -> sv_lang::Design {
    let session = Session::new();
    let mut comp = Compilation::new(&session).unwrap();
    comp.add_source(src).unwrap();
    comp.compile().unwrap()
}

#[test]
fn enum_base_members_and_values() {
    let design = compile(SRC);
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let t = body.find("c").unwrap().value_type().unwrap();
    assert!(t.is_enum(), "color_t is an enum (seen through the typedef)");
    assert!(!t.is_struct() && !t.is_array() && !t.is_union());

    // The underlying base type is `logic [1:0]`.
    let base = t.enum_base().expect("enum has a base type");
    assert_eq!(base.bit_width(), 2);

    // Members come back in declaration order.
    let members: Vec<_> = t.enum_members().collect();
    let names: Vec<_> = members.iter().map(|m| m.name().to_string()).collect();
    assert_eq!(names, ["RED", "GREEN", "BLUE"]);

    // Each member has a distinct printed constant value.
    let values: Vec<_> = members
        .iter()
        .map(|m| m.enum_member_value().expect("enum member has a value"))
        .collect();
    assert_eq!(values.len(), 3);
    assert!(!values[0].is_empty());
    assert_ne!(values[0], values[1]);
    assert_ne!(values[1], values[2]);
    assert_ne!(values[0], values[2]);
}

#[test]
fn struct_fields_and_offsets() {
    let design = compile(SRC);
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let t = body.find("b").unwrap().value_type().unwrap();
    assert!(
        t.is_struct(),
        "byte_t is a struct (seen through the typedef)"
    );
    assert!(!t.is_enum() && !t.is_array());

    let fields: Vec<_> = t.fields().collect();
    let names: Vec<_> = fields.iter().map(|f| f.name().to_string()).collect();
    assert_eq!(names, ["hi", "lo"]);

    // Field indices follow declaration order.
    assert_eq!(fields[0].field_index(), 0);
    assert_eq!(fields[1].field_index(), 1);

    // Each field's element type is 4 bits (reuses the value-type accessor).
    assert_eq!(fields[0].value_type().unwrap().bit_width(), 4);
    assert_eq!(fields[1].value_type().unwrap().bit_width(), 4);

    // In a packed struct the first-declared field occupies the high bits.
    assert_eq!(fields[0].field_bit_offset(), 4, "hi is the high nibble");
    assert_eq!(fields[1].field_bit_offset(), 0, "lo is the low nibble");
}

#[test]
fn array_element_type() {
    let design = compile(SRC);
    let body = design
        .top_instances()
        .next()
        .unwrap()
        .instance_body()
        .unwrap();

    let t = body.find("mem").unwrap().value_type().unwrap();
    assert!(t.is_array(), "mem is an unpacked array");
    let elem = t.element_type().expect("array has an element type");
    assert_eq!(elem.bit_width(), 8);

    // A non-array type yields None.
    let scalar = body.find("c").unwrap().value_type().unwrap();
    assert!(scalar.element_type().is_none());
}
