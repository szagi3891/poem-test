use poem_openapi::{Object, OneOf};

#[derive(Object, Debug, PartialEq)]
pub struct A {
    v1: i32,
    v2: String,
}

#[derive(Object, Debug, PartialEq)]
pub struct B {
    v4: String,
}


#[derive(OneOf, Debug, PartialEq)]
#[oai(property_name = "type")]
pub enum MyObj2 {
    A(A),
    B(B),
}
