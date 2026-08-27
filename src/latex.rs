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
