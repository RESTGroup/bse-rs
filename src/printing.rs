//! Helpers for printing pieces of basis sets

use crate::prelude::*;

/// Find how many spaces to put before a column of numbers so that all the
/// decimal points line up.
fn determine_leftpad(column: &[String], point_place: usize) -> Vec<usize> {
    column
        .iter()
        .map(|s| {
            let ndigit_left = s.find('.').unwrap_or(0);
            (point_place as i32 - 1 - ndigit_left as i32).max(0) as usize
        })
        .collect()
}

pub fn write_matrix(mat: &[Vec<String>], point_places: &[usize], convert_exp: bool) -> String {
    // Padding for the whole matrix
    let pad = mat.iter().zip(point_places).map(|(c, &p)| determine_leftpad(c, p)).collect_vec();

    // Use the transposes (easier to write out by row)
    let pad = misc::transpose_matrix(&pad);
    let mat = misc::transpose_matrix(mat);

    let mut lines: Vec<String> = vec![];
    for (r, row) in mat.iter().enumerate() {
        let mut line = String::new();
        for (c, s) in row.iter().enumerate() {
            let mut sp = pad[r][c] - line.len();
            // ensure at least one space, except for the beginning of the line
            if c > 0 {
                sp = sp.max(1);
            }
            line.push_str(&" ".repeat(sp));
            line.push_str(s);
        }
        lines.push(line);
    }

    let mut lines = lines.join("\n");
    if convert_exp {
        lines = lines.replace("E", "D").replace("e", "D");
    }
    lines
}
