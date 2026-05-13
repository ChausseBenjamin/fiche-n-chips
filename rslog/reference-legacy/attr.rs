use std::fmt::Display;

pub struct Attr {
	pub key: String,
	pub val: Box<dyn Display + Send + Sync>,
}

pub fn attr(
	key: impl Into<String>,
	val: impl Display + Send + Sync + 'static,
) -> Attr {
	Attr {
		key: key.into(),
		val: Box::new(val),
	}
}

#[macro_export]
macro_rules! attrs {
    () => {
        Vec::<Attr>::new()
    };

    ($($tt:tt)*) => {{
        let mut v = Vec::new();
        attrs!(@parse v, $($tt)*);
        v
    }};

    // key-value pair first
    (@parse $v:ident, $k:expr, $val:expr $(, $rest:tt)*) => {{
        $v.push($crate::attr($k, $val));
        attrs!(@parse $v $(, $rest)*);
    }};

    // single Attr
    (@parse $v:ident, $a:expr $(, $rest:tt)*) => {{
        $v.push($a);
        attrs!(@parse $v $(, $rest)*);
    }};

    (@parse $v:ident) => {};
}
