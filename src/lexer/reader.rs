use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use ruinous_util::span::{Location, Span};

use super::{error::FileError, state::Continuation};

pub struct CharReader<R: BufRead> {
    input: R,
    path: PathBuf,
}

impl CharReader<BufReader<File>> {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, FileError> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|err| FileError::file_open(path.to_owned(), err))?;
        let reader = BufReader::new(file);

        Ok(Self {
            input: reader,
            path: path.to_owned(),
        })
    }
}

impl<R: BufRead> CharReader<R> {
    pub fn read<Callback: FnMut(Span<char>) -> Continuation>(
        mut self,
        mut callback: Callback,
    ) -> Result<(), FileError> {
        let mut buffer = String::new();
        let mut location = Location::default();

        loop {
            buffer.clear();

            match self.input.read_line(&mut buffer) {
                Ok(0) => return Ok(()),
                Err(err) => return Err(FileError::file_read(self.path, err)),
                _ => (),
            }

            let mut handle_char = |char| {
                let start = location;
                location.advance();
                while let Continuation::Peek = callback(Span::new(start, location, char)) {}
            };

            for char in buffer.trim_end_matches(|c| c == '\n' || c == '\r').chars() {
                handle_char(char);
            }

            handle_char('\n');

            location.next_line();
        }
    }

    pub fn collect(self) -> Result<Vec<Span<char>>, FileError> {
        let mut vec = Vec::new();

        self.read(|c| {
            vec.push(c);
            Continuation::Consume
        })?;

        Ok(vec)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn span<D>(
        data: D,
        from_line: usize,
        from_col: usize,
        to_line: usize,
        to_col: usize,
    ) -> Span<D> {
        Span::new(
            Location::new(from_line, from_col),
            Location::new(to_line, to_col),
            data,
        )
    }

    fn reader_test(input: &str, expected: &[Span<char>]) {
        let reader = CharReader {
            input: std::io::Cursor::new(input),
            path: "".into(),
        };

        let result = reader.collect().unwrap();
        assert_eq!(&result, expected);
    }

    fn combined_test(
        input1: &str,
        expected1: &[Span<char>],
        input2: &str,
        expected2: &[Span<char>],
    ) {
        if input1.is_empty() {
            return reader_test(input2, expected2);
        } else if input2.is_empty() {
            return reader_test(input1, expected1);
        }

        let input = format!("{input1}{input2}");

        let mut expected = expected1.to_owned();
        let new_line = input1.ends_with(|c| c == '\n' || c == '\r');

        if !new_line {
            expected.pop();
        }

        let (line, col) = if let Some(last) = expected.last() {
            if new_line {
                (last.end.line + 1, 0)
            } else {
                (last.end.line, last.end.col)
            }
        } else {
            (0, 0)
        };

        for item in expected2 {
            if item.start.line == 0 {
                expected.push(span(
                    item.data,
                    item.start.line + line,
                    item.start.col + col,
                    item.end.line + line,
                    item.end.col + col,
                ));
            } else {
                expected.push(span(
                    item.data,
                    item.start.line + line,
                    item.start.col,
                    item.end.line + line,
                    item.end.col,
                ));
            }
        }

        reader_test(&input, &expected);
    }

    macro_rules! test_cases {
        ($($name:ident($input:expr, $expected:expr))+) => {
            test_cases!(expand_single $(($name, $input, $expected))+);
            test_cases!(expand_multi $(($name, $input, $expected))+);
        };

        (expand_single $(($name:ident, $input:expr, $expected:expr))+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<read_ $name>]() {
                        reader_test($input, $expected);
                    }
                }
            )+
        };

        (expand_multi ($name1:ident, $input1:expr, $expected1:expr)($name2:ident, $input2:expr, $expected2:expr)) => {
            test_cases!(expand_multi_cases ($name1, $input1, $expected1)($name2, $input2, $expected2));
        };

        (expand_multi ($name1:ident, $input1:expr, $expected1:expr)($name2:ident, $input2:expr, $expected2:expr)$(($names:ident, $inputs:expr, $expecteds:expr))+) => {
            test_cases!(expand_multi_cases ($name1, $input1, $expected1)($name2, $input2, $expected2)$(($names, $inputs, $expecteds))+);
            test_cases!(expand_multi ($name2, $input2, $expected2)$(($names, $inputs, $expecteds))+);
        };

        (expand_multi_cases ($name1:ident, $input1:expr, $expected1:expr)($name2:ident, $input2:expr, $expected2:expr)) => {
            test_cases!(expand_multi_case ($name1, $input1, $expected1)($name2, $input2, $expected2));
        };

        (expand_multi_cases ($name1:ident, $input1:expr, $expected1:expr)($name2:ident, $input2:expr, $expected2:expr)$(($names:ident, $inputs:expr, $expecteds:expr))+) => {
            test_cases!(expand_multi_case ($name1, $input1, $expected1)($name2, $input2, $expected2));
            test_cases!(expand_multi_cases ($name1, $input1, $expected1)$(($names, $inputs, $expecteds))+);
        };

        (expand_multi_case ($name1:ident, $input1:expr, $expected1:expr)($name2:ident, $input2:expr, $expected2:expr)) => {
            paste::paste! {
                #[test]
                fn [<read_ $name1 _then_ $name2>]() {
                    combined_test($input1, $expected1, $input2, $expected2)
                }

                #[test]
                fn [<read_ $name2 _then_ $name1>]() {
                    combined_test($input2, $expected2, $input1, $expected1)
                }
            }
        };
    }

    test_cases! {
        empty_input("", &[])
        single_space(" ", &[span(' ', 0, 0, 0, 1), span('\n', 0, 1, 0, 2)])
        single_tab("\t", &[span('\t', 0, 0, 0, 1), span('\n', 0, 1, 0, 2)])
        single_newline("\n", &[span('\n', 0, 0, 0, 1)])
        single_char("a", &[span('a', 0, 0, 0, 1), span('\n', 0, 1, 0, 2)])
        single_emojji("🐉", &[span('🐉', 0, 0, 0, 1), span('\n', 0, 1, 0, 2)])
        char_sequence(
            "a1b2c3",
            &[
                span('a', 0, 0, 0, 1),
                span('1', 0, 1, 0, 2),
                span('b', 0, 2, 0, 3),
                span('2', 0, 3, 0, 4),
                span('c', 0, 4, 0, 5),
                span('3', 0, 5, 0, 6),
                span('\n', 0, 6, 0, 7)])
        char_spaces(
            "a b c ",
            &[
                span('a', 0, 0, 0, 1),
                span(' ', 0, 1, 0, 2),
                span('b', 0, 2, 0, 3),
                span(' ', 0, 3, 0, 4),
                span('c', 0, 4, 0, 5),
                span(' ', 0, 5, 0, 6),
                span('\n', 0, 6, 0, 7)])
        char_tabs(
            "a\tb\tc\t",
            &[
                span('a', 0, 0, 0, 1),
                span('\t', 0, 1, 0, 2),
                span('b', 0, 2, 0, 3),
                span('\t', 0, 3, 0, 4),
                span('c', 0, 4, 0, 5),
                span('\t', 0, 5, 0, 6),
                span('\n', 0, 6, 0, 7)])
        char_lines(
            "a\nb\nc",
            &[
                span('a', 0, 0, 0, 1),
                span('\n', 0, 1, 0, 2),
                span('b', 1, 0, 1, 1),
                span('\n', 1, 1, 1, 2),
                span('c', 2, 0, 2, 1),
                span('\n', 2, 1, 2, 2)])
        emoji_lines(
            "🐉\n😛\n🍎",
            &[
                span('🐉', 0, 0, 0, 1),
                span('\n', 0, 1, 0, 2),
                span('😛', 1, 0, 1, 1),
                span('\n', 1, 1, 1, 2),
                span('🍎', 2, 0, 2, 1),
                span('\n', 2, 1, 2, 2)])
    }
}
