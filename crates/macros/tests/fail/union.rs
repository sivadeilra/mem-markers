use macros::FixedLayout;

#[derive(FixedLayout)]
#[repr(C)]
union Foo {
    a: u8,
    b: u8,
}

fn main() {}
