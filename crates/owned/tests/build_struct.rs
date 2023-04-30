#[owned::owned]
struct Struct<'a> {
    #[owned(ty = String)]
    a: &'a str,
}

#[owned::owned]
struct Unnamed<'a>(#[owned(ty = String)] &'a str);

#[owned::owned]
struct Empty;
