//! SAS Transport (XPT) file format reader and writer.
//!
//! This crate provides functionality to read and write SAS Transport v5 format files,
//! commonly used for SDTM datasets in regulatory submissions.

// TODO(docs): Add documentation for all public items (Phase 4 - PR-028)
#![allow(missing_docs)]

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{Context, Result, anyhow};

const RECORD_LEN: usize = 80;
const NAMESTR_LEN: usize = 140;

const LIBRARY_HEADER_PREFIX: &str = "HEADER RECORD*******LIBRARY HEADER RECORD!!!!!!!";
const MEMBER_HEADER_PREFIX: &str = "HEADER RECORD*******MEMBER  HEADER RECORD!!!!!!!";
const DSCRPTR_HEADER_PREFIX: &str = "HEADER RECORD*******DSCRPTR HEADER RECORD!!!!!!!";
const NAMESTR_HEADER_PREFIX: &str = "HEADER RECORD*******NAMESTR HEADER RECORD!!!!!!!";
const OBS_HEADER_PREFIX: &str = "HEADER RECORD*******OBS     HEADER RECORD!!!!!!!";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XptType {
    Num,
    Char,
}

#[derive(Debug, Clone, PartialEq)]
pub enum XptValue {
    Num(Option<f64>),
    Char(String),
}

#[derive(Debug, Clone)]
pub struct XptColumn {
    pub name: String,
    pub label: Option<String>,
    pub data_type: XptType,
    pub length: u16,
}

#[derive(Debug, Clone)]
pub struct XptDataset {
    pub name: String,
    pub label: Option<String>,
    pub columns: Vec<XptColumn>,
    pub rows: Vec<Vec<XptValue>>,
}

#[derive(Debug, Clone)]
pub struct XptWriterOptions {
    pub sas_version: String,
    pub os_name: String,
    pub created: String,
    pub modified: String,
    pub dataset_type: String,
    pub missing_numeric: MissingNumeric,
}

#[derive(Debug, Clone, Copy)]
pub enum MissingNumeric {
    Standard,
    Special(char),
}

impl Default for XptWriterOptions {
    fn default() -> Self {
        Self {
            sas_version: "9.4".to_string(),
            os_name: "RUST".to_string(),
            created: "01JAN70:00:00:00".to_string(),
            modified: "01JAN70:00:00:00".to_string(),
            dataset_type: String::new(),
            missing_numeric: MissingNumeric::Standard,
        }
    }
}

pub fn write_xpt(path: &Path, dataset: &XptDataset, options: &XptWriterOptions) -> Result<()> {
    validate_dataset(dataset)?;
    let file = File::create(path).with_context(|| format!("create {}", path.display()))?;
    let mut writer = std::io::BufWriter::new(file);

    writer.write_all(&build_fixed_header(LIBRARY_HEADER_PREFIX)?)?;
    writer.write_all(&build_library_header_data(options)?)?;
    writer.write_all(&pad_ascii(&options.modified, RECORD_LEN))?;

    writer.write_all(&build_member_header_record(NAMESTR_LEN)?)?;
    writer.write_all(&build_fixed_header(DSCRPTR_HEADER_PREFIX)?)?;
    writer.write_all(&build_member_header_data(dataset, options)?)?;
    writer.write_all(&build_member_header_second(dataset, options)?)?;

    writer.write_all(&build_namestr_header(dataset.columns.len())?)?;
    write_namestr_records(&mut writer, &dataset.columns)?;

    writer.write_all(&build_fixed_header(OBS_HEADER_PREFIX)?)?;
    write_observation_records(&mut writer, dataset, options)?;

    writer.flush().context("flush xpt")?;
    Ok(())
}

pub fn read_xpt(path: &Path) -> Result<XptDataset> {
    let mut file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .with_context(|| format!("read {}", path.display()))?;
    if data.len() < RECORD_LEN * 8 {
        return Err(anyhow!("xpt file too small"));
    }
    if data.len() % RECORD_LEN != 0 {
        return Err(anyhow!("xpt file length is not a multiple of 80"));
    }

    let mut offset = 0usize;
    let library_header = read_record(&data, offset)?;
    if !starts_with_prefix(library_header, LIBRARY_HEADER_PREFIX) {
        return Err(anyhow!("missing library header record"));
    }
    offset += RECORD_LEN;

    offset += RECORD_LEN * 2; // library real header + modified header

    let member_header = read_record(&data, offset)?;
    if !starts_with_prefix(member_header, MEMBER_HEADER_PREFIX) {
        return Err(anyhow!("missing member header record"));
    }
    let namestr_len = parse_namestr_len(member_header)?;
    offset += RECORD_LEN;

    let dscrptr = read_record(&data, offset)?;
    if !starts_with_prefix(dscrptr, DSCRPTR_HEADER_PREFIX) {
        return Err(anyhow!("missing descriptor header record"));
    }
    offset += RECORD_LEN;

    let member_data = read_record(&data, offset)?;
    let dataset_name = parse_member_dataset_name(member_data)?;
    offset += RECORD_LEN;

    let member_second = read_record(&data, offset)?;
    let dataset_label = parse_member_dataset_label(member_second);
    offset += RECORD_LEN;

    let namestr_header = read_record(&data, offset)?;
    if !starts_with_prefix(namestr_header, NAMESTR_HEADER_PREFIX) {
        return Err(anyhow!("missing namestr header record"));
    }
    let var_count = parse_namestr_count(namestr_header)?;
    offset += RECORD_LEN;

    let namestr_total = var_count
        .checked_mul(namestr_len)
        .ok_or_else(|| anyhow!("namestr size overflow"))?;
    let namestr_data = read_block(&data, offset, namestr_total)?;
    offset += namestr_total;
    offset = align_to_record(offset);

    let obs_header = read_record(&data, offset)?;
    if !starts_with_prefix(obs_header, OBS_HEADER_PREFIX) {
        return Err(anyhow!("missing obs header record"));
    }
    offset += RECORD_LEN;

    let columns = parse_namestr_records(namestr_data, var_count, namestr_len)?;
    let obs_len = observation_length(&columns)?;
    let rows = parse_observations(&data, offset, obs_len, &columns)?;

    Ok(XptDataset {
        name: dataset_name,
        label: dataset_label,
        columns,
        rows,
    })
}

fn validate_dataset(dataset: &XptDataset) -> Result<()> {
    let name = normalize_name(&dataset.name);
    if name.is_empty() || name.len() > 8 {
        return Err(anyhow!("dataset name must be 1-8 characters"));
    }
    let mut seen = BTreeSet::new();
    for column in &dataset.columns {
        let col_name = normalize_name(&column.name);
        if col_name.is_empty() || col_name.len() > 8 {
            return Err(anyhow!(
                "column name {} must be 1-8 characters",
                column.name
            ));
        }
        if !seen.insert(col_name.clone()) {
            return Err(anyhow!("duplicate column name {}", col_name));
        }
        if column.length == 0 {
            return Err(anyhow!("column {} has zero length", column.name));
        }
    }
    Ok(())
}

fn build_fixed_header(prefix: &str) -> Result<Vec<u8>> {
    if prefix.len() != 48 {
        return Err(anyhow!("header prefix must be 48 chars"));
    }
    let mut record = Vec::with_capacity(RECORD_LEN);
    record.extend_from_slice(prefix.as_bytes());
    record.extend_from_slice(&[b'0'; 30]);
    record.extend_from_slice(b"  ");
    Ok(record)
}

fn build_library_header_data(options: &XptWriterOptions) -> Result<Vec<u8>> {
    let mut record = Vec::with_capacity(RECORD_LEN);
    record.extend_from_slice(&pad_ascii("SAS", 8));
    record.extend_from_slice(&pad_ascii("SAS", 8));
    record.extend_from_slice(&pad_ascii("SASLIB", 8));
    record.extend_from_slice(&pad_ascii(&options.sas_version, 8));
    record.extend_from_slice(&pad_ascii(&options.os_name, 8));
    record.extend_from_slice(&[b' '; 24]);
    record.extend_from_slice(&pad_ascii(&options.created, 16));
    Ok(record)
}

fn build_member_header_record(namestr_len: usize) -> Result<Vec<u8>> {
    let mut record = build_fixed_header(MEMBER_HEADER_PREFIX)?;
    let size = format!("{:04}", 160);
    let namestr = format!("{:04}", namestr_len);
    for (idx, byte) in size.as_bytes().iter().enumerate() {
        record[64 + idx] = *byte;
    }
    for (idx, byte) in namestr.as_bytes().iter().enumerate() {
        record[74 + idx] = *byte;
    }
    Ok(record)
}

fn build_member_header_data(dataset: &XptDataset, options: &XptWriterOptions) -> Result<Vec<u8>> {
    let mut record = Vec::with_capacity(RECORD_LEN);
    record.extend_from_slice(&pad_ascii("SAS", 8));
    record.extend_from_slice(&pad_ascii(&normalize_name(&dataset.name), 8));
    record.extend_from_slice(&pad_ascii("SASDATA", 8));
    record.extend_from_slice(&pad_ascii(&options.sas_version, 8));
    record.extend_from_slice(&pad_ascii(&options.os_name, 8));
    record.extend_from_slice(&[b' '; 24]);
    record.extend_from_slice(&pad_ascii(&options.created, 16));
    Ok(record)
}

fn build_member_header_second(dataset: &XptDataset, options: &XptWriterOptions) -> Result<Vec<u8>> {
    let mut record = Vec::with_capacity(RECORD_LEN);
    record.extend_from_slice(&pad_ascii(&options.modified, 16));
    record.extend_from_slice(&[b' '; 16]);
    let label = dataset
        .label
        .clone()
        .unwrap_or_else(|| normalize_name(&dataset.name));
    record.extend_from_slice(&pad_ascii(&label, 40));
    record.extend_from_slice(&pad_ascii(&options.dataset_type, 8));
    Ok(record)
}

fn build_namestr_header(var_count: usize) -> Result<Vec<u8>> {
    let mut record = build_fixed_header(NAMESTR_HEADER_PREFIX)?;
    let count = format!("{:04}", var_count);
    for (idx, byte) in count.as_bytes().iter().enumerate() {
        record[54 + idx] = *byte;
    }
    Ok(record)
}

fn write_namestr_records<W: Write>(writer: &mut W, columns: &[XptColumn]) -> Result<()> {
    let mut record_writer = RecordWriter::new(writer);
    let mut offset = 0u32;
    for (idx, column) in columns.iter().enumerate() {
        let namestr = build_namestr(column, idx as u16 + 1, offset)?;
        record_writer.write_bytes(&namestr)?;
        offset = offset.saturating_add(column.length as u32);
    }
    record_writer.finish()?;
    Ok(())
}

fn build_namestr(column: &XptColumn, varnum: u16, pos: u32) -> Result<[u8; NAMESTR_LEN]> {
    let mut buf = [0u8; NAMESTR_LEN];
    let ntype = match column.data_type {
        XptType::Num => 1i16,
        XptType::Char => 2i16,
    };
    buf[0..2].copy_from_slice(&ntype.to_be_bytes());
    buf[2..4].copy_from_slice(&0i16.to_be_bytes());
    buf[4..6].copy_from_slice(&(column.length as i16).to_be_bytes());
    buf[6..8].copy_from_slice(&(varnum as i16).to_be_bytes());
    buf[8..16].copy_from_slice(&pad_ascii(&normalize_name(&column.name), 8));
    let label = column.label.clone().unwrap_or_default();
    buf[16..56].copy_from_slice(&pad_ascii(&label, 40));
    buf[56..64].copy_from_slice(&pad_ascii("", 8));
    buf[64..66].copy_from_slice(&0i16.to_be_bytes());
    buf[66..68].copy_from_slice(&0i16.to_be_bytes());
    buf[68..70].copy_from_slice(&0i16.to_be_bytes());
    buf[70..72].copy_from_slice(&[0u8; 2]);
    buf[72..80].copy_from_slice(&pad_ascii("", 8));
    buf[80..82].copy_from_slice(&0i16.to_be_bytes());
    buf[82..84].copy_from_slice(&0i16.to_be_bytes());
    buf[84..88].copy_from_slice(&(pos as i32).to_be_bytes());
    Ok(buf)
}

fn write_observation_records<W: Write>(
    writer: &mut W,
    dataset: &XptDataset,
    options: &XptWriterOptions,
) -> Result<()> {
    let obs_len = observation_length(&dataset.columns)?;
    let mut record_writer = RecordWriter::new(writer);
    for row in &dataset.rows {
        if row.len() != dataset.columns.len() {
            return Err(anyhow!("row length does not match columns"));
        }
        let mut obs = vec![b' '; obs_len];
        let mut pos = 0usize;
        for (value, column) in row.iter().zip(dataset.columns.iter()) {
            let bytes = match (value, column.data_type) {
                (XptValue::Char(value), XptType::Char) => encode_char(value, column.length),
                (XptValue::Num(value), XptType::Num) => {
                    encode_numeric(*value, column.length, options.missing_numeric)
                }
                (XptValue::Char(value), XptType::Num) => {
                    let parsed = value.trim().parse::<f64>().ok();
                    encode_numeric(parsed, column.length, options.missing_numeric)
                }
                (XptValue::Num(value), XptType::Char) => {
                    let rendered = value.map(format_numeric).unwrap_or_default();
                    encode_char(&rendered, column.length)
                }
            };
            let end = pos + bytes.len();
            obs[pos..end].copy_from_slice(&bytes);
            pos += column.length as usize;
        }
        record_writer.write_bytes(&obs)?;
    }
    record_writer.finish()?;
    Ok(())
}

fn parse_namestr_records(
    data: &[u8],
    var_count: usize,
    namestr_len: usize,
) -> Result<Vec<XptColumn>> {
    if namestr_len < 88 {
        return Err(anyhow!("namestr length too small"));
    }
    let mut columns = Vec::with_capacity(var_count);
    for idx in 0..var_count {
        let offset = idx
            .checked_mul(namestr_len)
            .ok_or_else(|| anyhow!("namestr offset overflow"))?;
        let record = data
            .get(offset..offset + namestr_len)
            .ok_or_else(|| anyhow!("namestr record out of bounds"))?;
        let ntype = read_i16(record, 0)?;
        let length = read_i16(record, 4)? as u16;
        let name = read_string(record, 8, 8);
        let label = read_string(record, 16, 40);
        columns.push(XptColumn {
            name,
            label: if label.is_empty() { None } else { Some(label) },
            data_type: if ntype == 1 {
                XptType::Num
            } else {
                XptType::Char
            },
            length,
        });
    }
    Ok(columns)
}

fn parse_observations(
    data: &[u8],
    offset: usize,
    obs_len: usize,
    columns: &[XptColumn],
) -> Result<Vec<Vec<XptValue>>> {
    if obs_len == 0 {
        return Ok(Vec::new());
    }
    if offset > data.len() {
        return Err(anyhow!("observation offset out of bounds"));
    }
    let data_len = data.len().saturating_sub(offset);
    let rows_total = data_len / obs_len;
    let remainder = data_len % obs_len;
    if remainder != 0 {
        let start = offset + rows_total * obs_len;
        let rem_bytes = &data[start..offset + data_len];
        if rem_bytes.iter().any(|b| *b != b' ') {
            return Err(anyhow!("unexpected trailing bytes in observations"));
        }
    }
    let mut rows = rows_total;
    while rows > 0 {
        let start = offset + (rows - 1) * obs_len;
        let row_bytes = &data[start..start + obs_len];
        if row_bytes.iter().all(|b| *b == b' ') {
            rows -= 1;
        } else {
            break;
        }
    }
    let mut output = Vec::with_capacity(rows);
    for row_idx in 0..rows {
        let start = offset + row_idx * obs_len;
        let row_bytes = &data[start..start + obs_len];
        let mut values = Vec::with_capacity(columns.len());
        let mut pos = 0usize;
        for column in columns {
            let len = column.length as usize;
            let slice = &row_bytes[pos..pos + len];
            let value = match column.data_type {
                XptType::Char => XptValue::Char(decode_char(slice)),
                XptType::Num => XptValue::Num(decode_numeric(slice)),
            };
            values.push(value);
            pos += len;
        }
        output.push(values);
    }
    Ok(output)
}

fn parse_member_dataset_name(record: &[u8]) -> Result<String> {
    if record.len() < 16 {
        return Err(anyhow!("member header too short"));
    }
    Ok(read_string(record, 8, 8))
}

fn parse_member_dataset_label(record: &[u8]) -> Option<String> {
    if record.len() < 72 {
        return None;
    }
    let label = read_string(record, 32, 40);
    if label.is_empty() { None } else { Some(label) }
}

fn parse_namestr_len(record: &[u8]) -> Result<usize> {
    let text = read_string(record, 74, 4);
    parse_usize(&text).context("parse namestr length")
}

fn parse_namestr_count(record: &[u8]) -> Result<usize> {
    let text = read_string(record, 54, 4);
    parse_usize(&text).context("parse namestr count")
}

fn observation_length(columns: &[XptColumn]) -> Result<usize> {
    let mut total = 0usize;
    for column in columns {
        total = total
            .checked_add(column.length as usize)
            .ok_or_else(|| anyhow!("observation length overflow"))?;
    }
    Ok(total)
}

fn read_record(data: &[u8], offset: usize) -> Result<&[u8]> {
    let slice = data
        .get(offset..offset + RECORD_LEN)
        .ok_or_else(|| anyhow!("record out of bounds"))?;
    Ok(slice)
}

fn read_block(data: &[u8], offset: usize, len: usize) -> Result<&[u8]> {
    let slice = data
        .get(offset..offset + len)
        .ok_or_else(|| anyhow!("block out of bounds"))?;
    Ok(slice)
}

fn align_to_record(offset: usize) -> usize {
    if offset.is_multiple_of(RECORD_LEN) {
        offset
    } else {
        offset + (RECORD_LEN - (offset % RECORD_LEN))
    }
}

fn starts_with_prefix(record: &[u8], prefix: &str) -> bool {
    record.starts_with(prefix.as_bytes())
}

fn read_i16(data: &[u8], offset: usize) -> Result<i16> {
    let bytes = data
        .get(offset..offset + 2)
        .ok_or_else(|| anyhow!("short out of bounds"))?;
    Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_string(data: &[u8], offset: usize, len: usize) -> String {
    data.get(offset..offset + len)
        .map(|slice| {
            let text = String::from_utf8_lossy(slice).to_string();
            text.trim().to_string()
        })
        .unwrap_or_default()
}

fn parse_usize(text: &str) -> Result<usize> {
    let digits = text.trim();
    if digits.is_empty() {
        return Err(anyhow!("empty numeric field"));
    }
    Ok(digits.parse::<usize>()?)
}

fn normalize_name(value: &str) -> String {
    let mut trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return trimmed;
    }
    trimmed.make_ascii_uppercase();
    trimmed
}

fn pad_ascii(value: &str, len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(len);
    for ch in value.chars().take(len) {
        if ch.is_ascii() {
            out.push(ch as u8);
        } else {
            out.push(b'?');
        }
    }
    while out.len() < len {
        out.push(b' ');
    }
    out
}

fn encode_char(value: &str, len: u16) -> Vec<u8> {
    pad_ascii(value, len as usize)
}

fn decode_char(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes).to_string();
    text.trim_end().to_string()
}

fn encode_numeric(value: Option<f64>, len: u16, missing: MissingNumeric) -> Vec<u8> {
    let mut bytes = [0u8; 8];
    match value {
        None => {
            bytes[0] = match missing {
                MissingNumeric::Standard => 0x2e,
                MissingNumeric::Special(code) => code as u8,
            };
        }
        Some(num) => {
            if !num.is_finite() {
                bytes[0] = 0x2e;
            } else {
                bytes = f64_to_ibm(num);
            }
        }
    }
    bytes[..len as usize].to_vec()
}

fn decode_numeric(bytes: &[u8]) -> Option<f64> {
    if bytes.is_empty() {
        return None;
    }
    let first = bytes[0];
    if is_missing_numeric(first, bytes) {
        return None;
    }
    let mut buf = [0u8; 8];
    let len = bytes.len().min(8);
    buf[..len].copy_from_slice(&bytes[..len]);
    Some(ibm_to_f64(buf))
}

fn is_missing_numeric(first: u8, bytes: &[u8]) -> bool {
    let rest_zero = bytes.iter().skip(1).all(|b| *b == 0);
    if !rest_zero {
        return false;
    }
    matches!(first, 0x2e | 0x5f | 0x41..=0x5a)
}

fn format_numeric(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        value.to_string()
    }
}

fn f64_to_ibm(value: f64) -> [u8; 8] {
    if value == 0.0 {
        return [0u8; 8];
    }
    let sign = if value < 0.0 { 0x80 } else { 0 };
    let mut v = value.abs();
    let mut exp: i32 = 64;
    while v < 0.0625 {
        v *= 16.0;
        exp -= 1;
    }
    while v >= 1.0 {
        v /= 16.0;
        exp += 1;
    }
    if exp <= 0 {
        return [0u8; 8];
    }
    if exp > 127 {
        return [0u8; 8];
    }
    let mut frac = (v * (1u64 << 56) as f64).round() as u64;
    if frac >= (1u64 << 56) {
        frac = (1u64 << 56) - 1;
    }
    let frac_bytes = frac.to_be_bytes();
    let mut out = [0u8; 8];
    out[0] = sign | (exp as u8 & 0x7f);
    out[1..].copy_from_slice(&frac_bytes[1..]);
    out
}

fn ibm_to_f64(bytes: [u8; 8]) -> f64 {
    if bytes.iter().all(|b| *b == 0) {
        return 0.0;
    }
    let sign = if bytes[0] & 0x80 != 0 { -1.0 } else { 1.0 };
    let exp = (bytes[0] & 0x7f) as i32;
    let frac = u64::from_be_bytes(bytes) & 0x00ff_ffff_ffff_ffff;
    let frac_value = (frac as f64) / (1u64 << 56) as f64;
    sign * 16f64.powi(exp - 64) * frac_value
}

struct RecordWriter<'a, W: Write> {
    writer: &'a mut W,
    record: [u8; RECORD_LEN],
    pos: usize,
}

impl<'a, W: Write> RecordWriter<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            record: [b' '; RECORD_LEN],
            pos: 0,
        }
    }

    fn write_bytes(&mut self, mut bytes: &[u8]) -> Result<()> {
        while !bytes.is_empty() {
            let remaining = RECORD_LEN - self.pos;
            let take = remaining.min(bytes.len());
            self.record[self.pos..self.pos + take].copy_from_slice(&bytes[..take]);
            self.pos += take;
            bytes = &bytes[take..];
            if self.pos == RECORD_LEN {
                self.writer.write_all(&self.record)?;
                self.record = [b' '; RECORD_LEN];
                self.pos = 0;
            }
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        if self.pos > 0 {
            for idx in self.pos..RECORD_LEN {
                self.record[idx] = b' ';
            }
            self.writer.write_all(&self.record)?;
            self.pos = 0;
        }
        Ok(())
    }
}
