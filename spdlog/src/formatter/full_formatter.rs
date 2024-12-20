//! Provides a full info formatter.

use std::fmt::{self, Write};

use cfg_if::cfg_if;

use crate::{
    formatter::{fmt_with_time, Formatter, FormatterContext, TimeDate},
    Error, Record, StringBuf, __EOL,
};

#[rustfmt::skip]
/// Full information logs formatter.
///
/// It is the default formatter for most sinks.
///
/// Log messages formatted by it look like:
///
///  - Default:
///
///    <pre>
///    [2022-11-02 09:23:12.263] [<font color="#0DBC79">info</font>] hello, world!
///    </pre>
///
///  - If the logger has a name:
///
///    <pre>
///    [2022-11-02 09:23:12.263] [logger-name] [<font color="#0DBC79">info</font>] hello, world!
///    </pre>
/// 
///  - If crate feature `source-location` is enabled:
///
///    <pre>
///    [2022-11-02 09:23:12.263] [logger-name] [<font color="#0DBC79">info</font>] [mod::path, src/main.rs:4] hello, world!
///    </pre>
#[derive(Clone)]
pub struct FullFormatter {
    with_eol: bool,
}

impl FullFormatter {
    /// Constructs a `FullFormatter`.
    #[must_use]
    pub fn new() -> FullFormatter {
        FullFormatter { with_eol: true }
    }

    #[must_use]
    pub(crate) fn without_eol() -> Self {
        Self { with_eol: false }
    }

    fn format_impl(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        ctx: &mut FormatterContext,
    ) -> Result<(), fmt::Error> {
        cfg_if! {
            if #[cfg(not(feature = "flexible-string"))] {
                dest.reserve(crate::string_buf::RESERVE_SIZE);
            }
        }

        fmt_with_time(ctx, record, |mut time: TimeDate| {
            dest.write_str("[")?;
            dest.write_str(time.full_second_str())?;
            dest.write_str(".")?;
            write!(dest, "{:03}", time.millisecond())?;
            dest.write_str("] [")?;
            Ok(())
        })?;

        if let Some(logger_name) = record.logger_name() {
            dest.write_str(logger_name)?;
            dest.write_str("] [")?;
        }

        let style_range_begin = dest.len();

        dest.write_str(record.level().as_str())?;

        let style_range_end = dest.len();

        if let Some(srcloc) = record.source_location() {
            dest.write_str("] [")?;
            dest.write_str(srcloc.module_path())?;
            dest.write_str(", ")?;
            dest.write_str(srcloc.file())?;
            dest.write_str(":")?;
            write!(dest, "{}", srcloc.line())?;
        }

        dest.write_str("] ")?;
        dest.write_str(record.payload())?;

        if self.with_eol {
            dest.write_str(__EOL)?;
        }

        ctx.set_style_range(Some(style_range_begin..style_range_end));
        Ok(())
    }
}

impl Formatter for FullFormatter {
    fn format(
        &self,
        record: &Record,
        dest: &mut StringBuf,
        ctx: &mut FormatterContext,
    ) -> crate::Result<()> {
        self.format_impl(record, dest, ctx)
            .map_err(Error::FormatRecord)
    }
}

impl Default for FullFormatter {
    fn default() -> FullFormatter {
        FullFormatter::new()
    }
}

#[cfg(test)]
mod tests {
    use chrono::prelude::*;

    use super::*;
    use crate::{Level, __EOL};

    #[test]
    fn format() {
        let record = Record::new(Level::Warn, "test log content", None, None);
        let mut buf = StringBuf::new();
        let mut ctx = FormatterContext::new();
        FullFormatter::new()
            .format(&record, &mut buf, &mut ctx)
            .unwrap();

        let local_time: DateTime<Local> = record.time().into();
        assert_eq!(
            format!(
                "[{}] [warn] test log content{}",
                local_time.format("%Y-%m-%d %H:%M:%S.%3f"),
                __EOL
            ),
            buf
        );
        assert_eq!(Some(27..31), ctx.style_range());
    }
}
