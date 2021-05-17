// Note that we use <$wrapper> which we do because the macro_rules takes wrapper as a type
// but to access the variant name we need a path and the way to convert a type to a path is by means of angle brackets
#[macro_export]
macro_rules! propagate {
    ($wrapper:ty, $variant:ident as $source:ty) => {
        impl std::convert::From<$source> for $wrapper {
            fn from(source: $source) -> Self {
                <$wrapper>::$variant(source)
            }
        }
    };
    ($wrapper:ty, $variant:ident as $source:ty, using_panic_feature) => {
        impl std::convert::From<$source> for $wrapper {
            fn from(source: $source) -> Self {
                if cfg!(feature = "panic_on_error_propagation") {
                    panic!("Paniced due to attempt to propagate: {}", source)
                } else {
                    <$wrapper>::$variant(source)
                }
            }
        }
    };
}
