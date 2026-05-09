# Slog

This is an attempt made by a Go user to try and feel like home when writing Rust.

This port aims to keep the spirit behind `log/slog` from the Go stdlib while
implementing it in the most idiomatic way for a Rust dev.


## Implementation decisions:

### LogValue

A log value is the "value" part of the saying "key/value pair" in
structured loggging. Not just *any* value. That being said, the only thing
required from it is the capability of displaying text
( implements `std::fmt::Display`). This then begs the question(s):

> Why is a custom datatype needed when a string can do this just find?

- Sometimes, it's better that what you log is different than the value itself.
  Example: It's useful to have full user credential to allow authentication but
           bad to log all credentials as it could leak sensitive data
- Giving any logging framework a pre-computed string implies you had to run some
  computation to read the string, even if in the end, that log statement is
  never captured.
  Example: Debug statements in an app need to print complex data types (which
  take time to compute), but the production deployment of the app only logs INFO
  level statements. Computation gets wasted generating strings that are never
  captured.

> How does this datatype fix these issues?

The answer is three-fold:

- a public `LogValuer` trait that is easy to implement
- clever use of macros
- defered evaluation

Any user can implement `LogValuer` for his own types to choose how log data is
formatted (ex: redacting sensitive data) but that method remains a second class
citizen. Then whenever the `log_val!` macro is used (done automatically when
calling higher-level macros), the custom `LogValuer` implementation for displaying
text is prioritized as the any `Display` that might exist, however the latter is
used as a fallback if no custom implementation is provided.

The handler within the logger has access to the function required to display
text, but it doesn't use it unless the global log level is below the threshold
of the statement. This lazy approach avoids unnecessary compute.

### Attrs

An attribute is the combination of "key" and "value" put together.
An enum was chosen as its type to allow for recursive data structures using
groups. `AttrLeaf` is the simplest attribute form containing only a key and a log
value.

Why are keys defined as `Arc<str>` and lists of attributes (in groups) as `Arc<[Attr]>`?

This decision was made based on how structured loggers are often used. Higher
level functions create their own logger and pass them to lower level functions
that fork them to append keys/values which can lead to keys/value pairs to exist
in many loggers and sub-loggers at once. Using `String` for keys would require a
copy of the vector to be made every time a logger that contains it gets forked.
The same rationale goes for grouped attributes: No need to duplicate data that
you do not want to mutate when forking loggers (to ensure consitency in logs).


### Level

A log level needs to achieve two things:

- Provide a human friendly way of discerning the importance of a log statement
- Provide a machine predictable way of disabling logs below a certain priority
  threshold

This library provides sane defaults and reasonable granularity through the
`SlogLevel` type. But, to accomodate users with needs beyond what is provided,
the `Level` trait is the only thing necessary for a custom implementation.

Why demand users implement `disp` and `value` instead of simply requiring the trait
also implements `Display` and `Ord`?

This decision was deliberately made as a consideration to the end-user writing logs
statement. Extending traits requires templating that could trickle all the way back
up to any log statements using those trait. <!-- TODO: fact check this... -->

Also, implementing `Ord`,`PartialOrd`,`Eq`,`PartialEq` can feel cumbersome to a
user that only wants a custom log-level type.

### Context

One of the most powerful features of the `log/slog` library in Go has to be it's
use of context. Through the use of custom handlers, a logger can be configured to
automatically pull relevant information about a request being made if it exists.

As Rust does not have make use of context in its most popular `http` frameworks
(axum and actix), an abstraction was made to simulate it over a variety of frameworks
that have different philosophies related to request management.

The trait is simple enough that it's easy to implement manually to lesser known
frameworks, but a ready-made implementation is accessible for both actix and axum
by enabling their respective features.

### Source

Handlers can be configured to show (or not) the location in code from which a
statement originates. In go, due to the limitations of the language, this source
location is evaluated at runtime. Rust does not do this natively, but with clever
use of macros, it can achieve better (compile-time evaluation).

