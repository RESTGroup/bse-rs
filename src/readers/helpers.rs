//! Some helper functions for parsing basis set files.

use crate::prelude::*;

lazy_static::lazy_static! {
    pub static ref FLOATING_RE: Regex = Regex::new(r"[-+]?\d*\.\d*(?:[dDeE][-+]?\d+)?").unwrap();
    pub static ref FLOATING_ONLY_RE: Regex = Regex::new(r"^[-+]?\d*\.\d*(?:[dDeE][-+]?\d+)?$").unwrap();
    pub static ref INTEGER_RE: Regex = Regex::new(r"[-+]?\d+").unwrap();
    pub static ref INTEGER_ONLY_RE: Regex = Regex::new(r"^[-+]?\d+$").unwrap();
    pub static ref BASIS_NAME_RE: Regex = Regex::new(r"\d*[a-zA-Z][a-zA-Z0-9\-\+\*\(\)\[\]]*").unwrap();
    pub static ref SPACES_RE: Regex = Regex::new(r"\s+").unwrap();
}

/// Tests if a string is a floating point number.
#[inline]
pub(crate) fn is_floating(s: &str) -> bool {
    replace_d(s).parse::<f64>().is_ok()
}

/// Tests if a string is an integer.
#[inline]
pub(crate) fn is_integer(s: &str) -> bool {
    s.parse::<i32>().is_ok()
}

/// Replace 'd' or 'D' with 'e' for scientific notation
#[inline]
pub(crate) fn replace_d(s: &str) -> String {
    s.replace('d', "e").replace('D', "E")
}

/// Creates a canonical list of AM for use with ECP potentials.
///
/// The list is [max_am, 0, 1, ..., max_am-1].
#[inline]
#[allow(unused)]
pub(crate) fn potential_am_list(max_am: i32) -> Vec<i32> {
    [max_am].into_iter().chain(0..max_am).collect()
}

/// Turns a list into a matrix of the given dimensions.
#[allow(unused)]
pub(crate) fn chunk_list<T: Clone>(lst: &[T], rows: usize, cols: usize) -> Result<Vec<Vec<T>>, BseError> {
    let n_elements = lst.len();
    if n_elements != rows * cols {
        bse_raise!(ValueError, "Cannot partition {n_elements} elements into a {rows}x{cols} matrix")?
    }
    let mat: Vec<Vec<T>> = lst.chunks(cols).map(|chunk| chunk.to_vec()).collect();
    debug_assert_eq!(mat.len(), rows);
    Ok(mat)
}

/// Tests the first element of the list to see if it is an expected string, and
/// removes it.
///
/// If line does not match, or lines is empty, an exception is raised.
#[allow(unused)]
pub(crate) fn remove_expected_line(lines: &[String], expected: &str, position: isize) -> Result<Vec<String>, BseError> {
    if lines.is_empty() {
        bse_raise!(ValueError, "No lines to test for expected line")?
    }
    if position >= 0 && lines.len() <= position as usize {
        bse_raise!(ValueError, "Not enough lines. Can't test line {position} when there are {} lines", lines.len())?
    } else if position < 0 && lines.len() < (-position) as usize {
        bse_raise!(ValueError, "Not enough lines. Can't test line {position} when there are {} lines", lines.len())?
    }

    let pos = if position >= 0 { position as usize } else { (lines.len() as isize + position) as usize };
    if lines[pos] != expected {
        bse_raise!(ValueError, "Expected line '{expected}' at position {pos}, but found '{}'", lines[pos])?
    }
    let mut new_lines = lines.to_vec();
    new_lines.remove(pos);
    Ok(new_lines)
}

pub fn parse_line_regex(rex: &Regex, line: &str, description: &str) -> Result<Vec<String>, BseError> {
    let captures = rex.captures(line).map_or(
        match description {
            "" => bse_raise!(ValueError, "Regex '{rex}' does not match line: '{line}'"),
            _ => bse_raise!(ValueError, "Regex '{description}' does not match line: '{line}'. Regex is '{rex}'"),
        },
        Ok,
    )?;

    let result: Vec<String> = captures
        .iter()
        .skip(1) // Skip the full match (group 0)
        .map(|m| m.map(|m| m.as_str().to_string()).unwrap_or_default())
        .collect();

    Ok(result)
}

pub fn parse_line_regex_dict(rex: &Regex, line: &str, description: &str) -> Result<HashMap<String, String>, BseError> {
    let captures = rex.captures(line).map_or(
        match description {
            "" => bse_raise!(ValueError, "Regex '{rex}' does not match line: '{line}'"),
            _ => bse_raise!(ValueError, "Regex '{description}' does not match line: '{line}'. Regex is '{rex}'"),
        },
        Ok,
    )?;

    let mut result = HashMap::new();
    for name in rex.capture_names().flatten() {
        if let Some(matches) = captures.name(name) {
            result.insert(name.to_string(), matches.as_str().to_string());
        }
    }

    Ok(result)
}

/// Partition a list of lines based on some condition
///
/// # Arguments
/// * `lines` - List of strings representing the lines in the file
/// * `condition` - Function that takes a line as an argument and returns true
///   if that line is the start of a section
/// * `before` - Number of lines prior to the splitting line to include
/// * `min_after` - Minimum number of lines to include after the match
/// * `min_blocks` - Minimum number of blocks to find
/// * `max_blocks` - Maximum number of blocks to find
/// * `min_size` - Minimum size/length of each block
/// * `include_match` - Whether to include the matching line in the block
pub fn partition_lines(
    lines: &[String],
    condition: impl Fn(&str) -> bool,
    before: usize,
    min_after: Option<usize>,
    min_blocks: Option<usize>,
    max_blocks: Option<usize>,
    min_size: usize,
    include_match: bool,
) -> Result<Vec<Vec<String>>, BseError> {
    // First, partition into blocks
    let mut all_blocks = Vec::new();
    let mut cur_block = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = &lines[i];
        if condition(line) {
            // New block found
            if !cur_block.is_empty() {
                all_blocks.push(cur_block);
                cur_block = Vec::new();
            }

            if include_match {
                cur_block.push(line.clone());
            }
            if let Some(after) = min_after {
                let end = std::cmp::min(i + 1 + after, lines.len());
                cur_block.extend(lines[i + 1..end].iter().cloned());
                i += after;
            }
        } else {
            cur_block.push(line.clone());
        }
        i += 1;
    }

    // Add last block if not empty
    if !cur_block.is_empty() {
        all_blocks.push(cur_block);
    }

    // Handle 'before' parameter
    if before > 0 {
        if all_blocks.len() <= 1 {
            bse_raise!(ValueError, "Cannot partition lines with before = {}: have {} blocks", before, all_blocks.len())?
        }

        if all_blocks[0].len() != before {
            bse_raise!(
                ValueError,
                "Cannot partition lines with before = {}: first block has {} lines",
                before,
                all_blocks[0].len()
            )?
        }

        // Move 'before' lines between blocks
        for idx in 1..all_blocks.len() {
            let split_point = all_blocks[idx - 1].len() - before;
            let mut moved = all_blocks[idx - 1].split_off(split_point);
            moved.append(&mut all_blocks[idx]);
            all_blocks[idx] = moved;
        }

        // Remove first block which should now be empty
        let first_block = all_blocks.remove(0);
        debug_assert!(first_block.is_empty());
    }

    // Validate blocks
    if min_size > 0 {
        for (idx, block) in all_blocks.iter().enumerate() {
            if block.len() < min_size {
                bse_raise!(ValueError, "Block {idx} does not have minimum number of lines ({min_size})")?
            }
        }
    }

    if let Some(min_blocks) = min_blocks {
        if all_blocks.len() < min_blocks {
            bse_raise!(ValueError, "Found {} blocks, but need at least {}", all_blocks.len(), min_blocks)?
        }
    }

    if let Some(max_blocks) = max_blocks {
        if all_blocks.len() > max_blocks {
            bse_raise!(ValueError, "Found {} blocks, but need at most {}", all_blocks.len(), max_blocks)?
        }
    }

    Ok(all_blocks)
}

/// Reads in a number of space-separated floating-point numbers
///
/// These numbers may span multiple lines.
///
/// Returns the found floating point numbers (as Strings), and the remaining
/// lines
pub fn read_n_floats(
    lines: &[String],
    n_numbers: usize,
    split_re: Option<&Regex>,
) -> Result<(Vec<String>, Vec<String>), BseError> {
    let split_re = split_re.unwrap_or(&SPACES_RE);
    let mut found_numbers = Vec::new();
    let mut remaining_lines = lines.to_vec();

    while found_numbers.len() < n_numbers {
        if remaining_lines.is_empty() {
            bse_raise!(ValueError, "Wanted {} numbers but ran out of lines after {}", n_numbers, found_numbers.len())?
        }

        let first_line = &remaining_lines[0];
        if first_line.trim().is_empty() {
            bse_raise!(ValueError, "Wanted {} numbers but found empty line after {}", n_numbers, found_numbers.len())?
        }

        let line = replace_d(first_line);
        let parts = split_re.split(line.trim()).filter(|s| !s.is_empty()).map(|s| s.to_string()).collect_vec();

        found_numbers.extend(parts);
        remaining_lines.remove(0);
    }

    if found_numbers.len() > n_numbers {
        bse_raise!(ValueError, "Wanted {n_numbers} numbers, but found extra numbers: {found_numbers:?}")?
    }

    // Verify all are floating point numbers
    if !found_numbers.iter().all(|x| is_floating(x)) {
        bse_raise!(ValueError, "Non-floating-point value found in numbers: {found_numbers:?}")?
    }

    Ok((found_numbers, remaining_lines))
}

/// Reads in all floats on all lines
///
/// This function takes a block of numbers and splits them all, for all lines in
/// the block. Returns the found floating point numbers as Strings.
pub fn read_all_floats(lines: &[String], split_re: Option<&Regex>) -> Result<Vec<String>, BseError> {
    let split_re = split_re.unwrap_or(&SPACES_RE);
    let found_numbers: Vec<String> = lines
        .iter()
        .flat_map(|line| {
            let processed_line = replace_d(line);
            split_re.split(processed_line.trim()).filter(|s| !s.is_empty()).map(|s| s.to_string()).collect_vec()
        })
        .collect();

    // Verify all are floating point numbers
    if !found_numbers.iter().all(|s| is_floating(s)) {
        bse_raise!(ValueError, "Non-floating-point value found in numbers: {:?}", found_numbers)?
    }

    Ok(found_numbers)
}

/// Reads in a number of space-separated integers
///
/// These numbers may span multiple lines.
/// Returns the found integers (as Strings) and the remaining lines
pub fn read_n_integers(
    lines: &[String],
    n_ints: usize,
    split_re: Option<&Regex>,
) -> Result<(Vec<String>, Vec<String>), BseError> {
    let split_re = split_re.unwrap_or(&SPACES_RE);
    let mut found_numbers = Vec::new();
    let mut remaining_lines = lines.to_vec();

    while found_numbers.len() < n_ints {
        if remaining_lines.is_empty() {
            bse_raise!(ValueError, "Wanted {n_ints} integers but ran out of lines after {}", found_numbers.len())?
        }

        let line = remaining_lines[0].trim();
        let parts: Vec<String> = split_re.split(line).filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();

        found_numbers.extend(parts);
        remaining_lines.remove(0);
    }

    if found_numbers.len() > n_ints {
        bse_raise!(ValueError, "Wanted {n_ints} integers, but found extra numbers: {found_numbers:?}")?
    }

    // Verify all are integers
    if !found_numbers.iter().all(|x| is_integer(x)) {
        bse_raise!(ValueError, "Non-integer value found in numbers: {found_numbers:?}")?
    }

    Ok((found_numbers, remaining_lines))
}

/// Parses a simple matrix of numbers with a predefined number of rows/columns
///
/// This will read in a matrix of the given number of rows and columns, even if
/// the rows span multiple lines. There must be a newline at the very end of a
/// row.
///
/// Returns the matrix and the remaining lines
pub fn parse_fixed_matrix(
    lines: &[String],
    rows: usize,
    cols: usize,
    split_re: Option<&Regex>,
) -> Result<(Vec<Vec<String>>, Vec<String>), BseError> {
    let split_re = split_re.unwrap_or(&SPACES_RE);
    let mut matrix = Vec::with_capacity(rows);
    let mut remaining_lines = lines.to_vec();

    for _ in 0..rows {
        let (row_data, new_lines) = read_n_floats(&remaining_lines, cols, Some(split_re))?;
        matrix.push(row_data);
        remaining_lines = new_lines;
    }

    Ok((matrix, remaining_lines))
}

/// Parses a simple matrix of numbers
///
/// The lines parameter must specify a list of strings containing the entire
/// matrix.
///
/// If rows and/or cols is specified, and the found number of rows/cols does not
/// match, an error is returned.
pub fn parse_matrix(
    lines: &[String],
    rows: Option<usize>,
    cols: Option<usize>,
    split_re: Option<&Regex>,
) -> Result<Vec<Vec<String>>, BseError> {
    let split_re = split_re.unwrap_or(&SPACES_RE);
    let mut mat = Vec::new();

    for line in lines {
        let processed_line = replace_d(line);
        let row: Vec<String> =
            split_re.split(processed_line.trim()).filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();

        if !row.iter().all(|x| is_floating(x)) {
            bse_raise!(ValueError, "Non-floating-point value found in matrix: {row:?}")?
        }

        if !row.is_empty() {
            mat.push(row);
        }
    }

    // Check matrix consistency
    if mat.is_empty() {
        bse_raise!(ValueError, "Empty matrix")?
    }

    let first_row_len = mat[0].len();
    if first_row_len == 0 {
        bse_raise!(ValueError, "Matrix row has zero values")?
    }

    for row in &mat {
        if row.len() != first_row_len {
            bse_raise!(ValueError, "Inconsistent number of columns: {} vs {}", row.len(), first_row_len)?
        }
    }

    // Validate dimensions if specified
    if let Some(expected_rows) = rows {
        if mat.len() != expected_rows {
            bse_raise!(ValueError, "Inconsistent number of rows: {expected_rows} vs {}", mat.len())?
        }
    }

    if let Some(expected_cols) = cols {
        if mat[0].len() != expected_cols {
            bse_raise!(ValueError, "Inconsistent number of columns: {expected_cols} vs {}", mat[0].len())?
        }
    }

    Ok(mat)
}

/// Parses a matrix/table of exponents and coefficients
///
/// The first column of the matrix contains exponents, and the remaining
/// columns contain the coefficients for all general contractions.
///
/// If nprim and/or ngen are specified, and the found number of
/// primitives/contractions doesn't match, an error is returned.
pub fn parse_primitive_matrix(
    lines: &[String],
    nprim: Option<usize>,
    ngen: Option<usize>,
    split_re: Option<&Regex>,
) -> Result<(Vec<String>, Vec<Vec<String>>), BseError> {
    let split_re = split_re.unwrap_or(&SPACES_RE);
    let mut exponents = Vec::new();
    let mut coefficients = Vec::new();

    for line in lines {
        let processed_line = replace_d(line);
        let parts: Vec<String> =
            split_re.split(processed_line.trim()).filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();

        if parts.is_empty() {
            continue;
        }

        let e = parts[0].clone();
        let c = parts[1..].to_vec();

        if !is_floating(&e) {
            bse_raise!(ValueError, "Non-floating-point value found in exponents: {e}")?
        }

        if !c.iter().all(|x| is_floating(x)) {
            bse_raise!(ValueError, "Non-floating-point value found in coefficients: {c:?}")?
        }

        exponents.push(e);
        coefficients.push(c);
    }

    // Validate coefficients structure
    if coefficients.is_empty() && !exponents.is_empty() {
        bse_raise!(ValueError, "Missing contraction coefficients")?
    }

    let first_coeff_len = coefficients.first().map_or(0, |v| v.len());
    for (i, c) in coefficients.iter().enumerate() {
        if c.is_empty() {
            bse_raise!(ValueError, "Missing contraction coefficients in row {}", i + 1)?
        }
        if c.len() != first_coeff_len {
            bse_raise!(ValueError, "Inconsistent number of coefficients: {} vs {}", c.len(), first_coeff_len)?
        }
    }

    let coefficients = misc::transpose_matrix(&coefficients);

    // Validate matrix structure
    if exponents.is_empty() {
        bse_raise!(ValueError, "No exponents found")?
    }
    if coefficients.is_empty() {
        bse_raise!(ValueError, "No coefficients found")?
    }

    // Validate dimensions if specified
    if let Some(expected_nprim) = nprim {
        if exponents.len() != expected_nprim {
            bse_raise!(
                ValueError,
                "Inconsistent number of primitives in exponents: {expected_nprim} vs {}",
                exponents.len()
            )?
        }

        if coefficients[0].len() != expected_nprim {
            bse_raise!(
                ValueError,
                "Inconsistent number of primitives in coefficients: {expected_nprim} vs {}",
                coefficients[0].len()
            )?
        }
    }

    if let Some(expected_ngen) = ngen {
        if coefficients.len() != expected_ngen {
            bse_raise!(
                ValueError,
                "Inconsistent number of general contractions: {expected_ngen} vs {}",
                coefficients.len()
            )?
        }
    }

    Ok((exponents, coefficients))
}

pub struct ReaderECP {
    pub r_exp: Vec<i32>,
    pub g_exp: Vec<String>,
    pub coeff: Vec<Vec<String>>,
}

/// Parses an ECP (Effective Core Potential) table from input lines
///
/// # Arguments
/// * `lines` - Input lines containing the ECP data
/// * `order` - Order of columns in the input (e.g., &["r_exp", "g_exp",
///   "coeff"])
/// * `split` - Regex pattern for splitting columns
///
/// # Returns
/// `ReaderECP` struct containing the parsed data
pub fn parse_ecp_table(lines: &[String], order: &[&str], split_re: Option<&Regex>) -> Result<ReaderECP, BseError> {
    let split_re = split_re.unwrap_or(&SPACES_RE);
    // Validate column order
    if order.len() != 3 {
        bse_raise!(ValueError, "ECP table requires exactly 3 columns, got {}", order.len())?
    }

    let mut r_exp = Vec::new();
    let mut g_exp = Vec::new();
    let mut coeff = Vec::new();

    for line in lines {
        let processed_line = replace_d(line);
        let parts: Vec<String> =
            split_re.split(processed_line.trim()).filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();

        if parts.len() != 3 {
            bse_raise!(ValueError, "Expected 3 values in ECP table, found {}", parts.len())?
        }

        // Map columns based on order
        let mut columns = HashMap::new();
        for (i, &key) in order.iter().enumerate() {
            columns.insert(key, parts[i].clone());
        }

        r_exp.push(columns["r_exp"].clone());
        g_exp.push(columns["g_exp"].clone());
        // Note: This function does not handle multiple coefficients
        // So we only have to add another layer to the coefficient list
        coeff.push(columns["coeff"].clone());
    }

    // Validate data types
    if !r_exp.iter().all(|x| is_integer(x)) {
        bse_raise!(ValueError, "Non-integer value found in r exponents: {:?}", r_exp)?
    }

    if !g_exp.iter().all(|x| is_floating(x)) {
        bse_raise!(ValueError, "Non-floating-point value found in g exponents: {:?}", g_exp)?
    }

    if !coeff.iter().all(|x| is_floating(x)) {
        bse_raise!(ValueError, "Non-floating-point value found in coefficients: {:?}", coeff)?
    }

    // Convert r exponents to integers
    let r_exp = r_exp.into_iter().map(|x| x.parse().unwrap()).collect_vec();

    Ok(ReaderECP { r_exp, g_exp, coeff: vec![coeff] })
}

/// Remove comment and blank lines
///
/// Also strips all lines of beginning/ending whitespace.
///
/// # Arguments
/// * `lines` - List of lines to prune
/// * `skipchars` - Comment characters designating lines to be skipped
/// * `prune_blank` - Remove blank lines
/// * `strip_end_blanks` - Remove starting/ending blank lines (even if
///   prune_blank=false)
pub fn prune_lines(lines: &[String], skipchars: &str, prune_blank: bool, strip_end_blanks: bool) -> Vec<String> {
    let mut processed: Vec<String> = lines.iter().map(|l| l.trim().to_string()).collect();

    // Filter comments if skipchars specified
    if !skipchars.is_empty() {
        processed.retain(|l| l.is_empty() || !skipchars.contains(l.chars().next().unwrap_or(' ')));
    }

    // Filter blank lines if requested
    if prune_blank {
        processed.retain(|l| !l.is_empty());
    }

    // Early return if empty
    if processed.is_empty() {
        return processed;
    }

    // Strip end blanks if requested and we didn't already prune all blanks
    if strip_end_blanks && !prune_blank {
        // Remove leading blanks
        while !processed.is_empty() && processed[0].is_empty() {
            processed.remove(0);
        }
        // Remove trailing blanks
        while !processed.is_empty() && processed.last().unwrap().is_empty() {
            processed.pop();
        }
    }

    processed
}

/// Removes a block of data from the lines of text
///
/// For example, there may be an optional block of options (like in molcas)
/// This will only remove a single block
///
/// # Arguments
/// * `lines` - Lines of text to parse
/// * `start_re` - Regex string representing the start of the block (case
///   insensitive)
/// * `end_re` - Regex string representing the end of the block (case
///   insensitive)
///
/// # Returns
/// `(Vec<String>, Vec<String>)` - The block found (may be empty), the input
/// lines without the block
pub fn remove_block(
    lines: &[String],
    start_re: &Regex,
    end_re: &Regex,
) -> Result<(Vec<String>, Vec<String>), BseError> {
    let mut start_idx = None;
    for (idx, line) in lines.iter().enumerate() {
        if start_re.is_match(line) {
            if start_idx.is_some() {
                bse_raise!(ValueError, "Multiple blocks starting with '{}' found", start_re)?
            }
            start_idx = Some(idx);
            break;
        }
    }

    let start_idx = match start_idx {
        Some(idx) => idx,
        None => return Ok((Vec::new(), lines.to_vec())), // Block not found
    };

    let mut block_lines = Vec::new();
    let mut i = start_idx + 1;
    while i < lines.len() && !end_re.is_match(&lines[i]) {
        block_lines.push(lines[i].clone());
        i += 1;
    }

    // Check if we found the end of the block
    if i == lines.len() {
        bse_raise!(ValueError, "Cannot find end of block. Looking for '{end_re}' to close '{start_re}'")?
    }

    let mut remaining_lines = lines[..start_idx].to_vec();
    remaining_lines.extend_from_slice(&lines[i + 1..]);

    Ok((block_lines, remaining_lines))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playground_line_regex() {
        let rex = Regex::new(r"^(?P<sym>[A-Za-z]+)\s+(?P<name>\d+)((?:\s+)+)$").unwrap();
        let line = "H 1    ";
        let res = parse_line_regex(&rex, line, "Test regex parsing").unwrap();
        println!("{res:?}");
        let res = parse_line_regex_dict(&rex, line, "Test regex parsing").unwrap();
        println!("{res:?}");
    }
}
