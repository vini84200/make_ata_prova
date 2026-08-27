use askama::filters::Escaper;
use std::fmt::Write;

#[derive(Debug, Copy, Clone)]
pub struct Tex;

impl Escaper for Tex {
    fn write_escaped_str<W: Write>(&self, mut dest: W, string: &str) -> core::fmt::Result {
        for c in string.chars() {
            match c {
                '\\' => dest.write_str(r"\textbackslash{}")?,
                '{'  => dest.write_str(r"\{")?,
                '}'  => dest.write_str(r"\}")?,
                '_'  => dest.write_str(r"\_")?,
                '^'  => dest.write_str(r"\textasciicircum{}")?,
                '%'  => dest.write_str(r"\%")?,
                '&'  => dest.write_str(r"\&")?,
                '#'  => dest.write_str(r"\#")?,
                '$'  => dest.write_str(r"\$")?,
                '~'  => dest.write_str(r"\textasciitilde{}")?,
                _    => dest.write_char(c)?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn escape(input: &str) -> String {
        let mut out = String::new();
        Tex.write_escaped_str(&mut out, input).unwrap();
        out
    }

    #[test]
    fn escapes_each_special_character() {
        assert_eq!(escape(r"\"), r"\textbackslash{}");
        assert_eq!(escape("{"), r"\{");
        assert_eq!(escape("}"), r"\}");
        assert_eq!(escape("_"), r"\_");
        assert_eq!(escape("^"), r"\textasciicircum{}");
        assert_eq!(escape("%"), r"\%");
        assert_eq!(escape("&"), r"\&");
        assert_eq!(escape("#"), r"\#");
        assert_eq!(escape("$"), r"\$");
        assert_eq!(escape("~"), r"\textasciitilde{}");
    }

    #[test]
    fn leaves_plain_text_untouched() {
        assert_eq!(escape("Maria Souza"), "Maria Souza");
    }

    #[test]
    fn escapes_a_mix_within_a_sentence() {
        assert_eq!(
            escape("50% de desconto para turma A_1 & B#2"),
            r"50\% de desconto para turma A\_1 \& B\#2"
        );
    }
}
