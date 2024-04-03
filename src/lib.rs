use binrw::{helpers, io::TakeSeekExt, prelude::*, NullString, PosValue};
use std::io::{Cursor, Read, Seek, SeekFrom};

#[binread]
#[derive(Debug)]
pub struct Avi {
    #[br(count = 4, map = vec_u8_to_string)]
    pub id: String,
    pub size: i32,
    #[br(count = 4, map = vec_u8_to_string)]
    pub kind: String,
    #[br(map_stream = |reader| reader.take_seek(size as u64 - 4), parse_with = helpers::until_eof)]
    pub chunks: Vec<Chunk>,
}

fn vec_u8_to_string(x: Vec<u8>) -> String {
    String::from_utf8(x).unwrap()
}

#[binread]
#[derive(Debug)]
#[br(import(index_entry: &IndexEntry, movi_offset: u64))]
pub struct Frame(
    #[br(seek_before = SeekFrom::Start(index_entry.dw_chunk_offset as u64 + movi_offset + 0x10))]
    #[br(count = index_entry.dw_chunk_length)]
    pub Vec<u8>,
);

#[binread]
#[derive(Debug)]
pub struct Chunk {
    pub offset: PosValue<()>,
    #[br(count = 4, map = vec_u8_to_string)]
    pub id: String,
    pub size: i32,
    #[br(pad_size_to = size)]
    #[br(args(&id, size))]
    pub kind: ChunkType,
}

#[binread]
#[derive(Debug, PartialEq)]
#[br(import(id: &str, size: i32))]
pub enum ChunkType {
    #[br(magic = b"hdrl")]
    Hdrl {
        avi_header: AviHeader,
        #[br(count = 2)]
        chunks: Vec<ListInfoChunk>,
    },
    #[br(magic = b"INFO")]
    Info,
    #[br(magic = b"movi")]
    Movi,
    #[br(pre_assert(id == "idx1"))]
    Idx1 {
        #[br(count = size / 16)]
        index_entries: Vec<IndexEntry>,
    },
    Unknown,
}

#[binread]
#[derive(Debug, PartialEq)]
pub struct ListInfoChunk {
    #[br(count = 4, map = vec_u8_to_string)]
    pub id: String,
    pub size: i32,
    #[br(count = 4, map = vec_u8_to_string)]
    pub kind: String,
    #[br(map_stream = |reader| reader.take_seek(size as u64 - 4), parse_with = helpers::until_eof)]
    pub header: Vec<InfoChunk>,
}

#[binread]
#[derive(Debug, PartialEq)]
pub enum InfoChunk {
    #[brw(magic = b"strh")]
    StreamHeader(#[br(temp)] u32, #[br(pad_size_to = self_0)] StreamHeader),
    #[brw(magic = b"strf")]
    StreamFormat(#[br(temp)] u32, #[br(pad_size_to = self_0)] StreamFormat),
    #[brw(magic = b"strn")]
    StreamName(#[br(temp)] u32, #[br(pad_size_to = self_0 + 1)] NullString),
    Unknown {
        #[br(count = 4, map = vec_u8_to_string)]
        id: String,
        size: u32,
        // #[br(temp, count = size)]
        // data: Vec<u8>,
        #[br(pad_size_to = size)]
        nothing: (),
    },
}

#[binread]
#[derive(Debug, PartialEq)]
#[br(magic = b"avih")]
pub struct AviHeader {
    pub size: u32,
    pub time_between_frames: i32,
    pub maxium_data_rate: i32,
    pub padding_granularity: i32,
    pub flags: i32,
    pub total_number_of_frames: i32,
    pub number_of_initial_frames: i32,
    pub number_of_streams: i32,
    pub suggested_buffer_size: i32,
    pub width: i32,
    pub height: i32,
    pub time_scale: i32,
    pub data_rate: i32,
    pub start_time: i32,
    pub data_length: i32,
}

impl AviHeader {
    pub fn fps(&self) -> f32 {
        self.time_between_frames as f32 * 0.000001
    }
}

#[binread]
#[derive(Debug, PartialEq)]
pub struct StreamHeader {
    #[br(count = 4, map = vec_u8_to_string)]
    pub data_type: String,
    #[br(count = 4, map = vec_u8_to_string)]
    pub data_handler: String,
    pub flags: i32,
    pub priority: i32,
    pub initial_frames: i32,
    pub time_scale: i32,
    pub data_rate: i32,
    pub start_time: i32,
    pub data_length: i32,
    pub suggested_buffer_size: i32,
    pub quality: i32,
    pub sample_size: i32,
}

#[binread]
#[derive(Debug, PartialEq)]
pub struct StreamFormat {
    pub header_size: i32,
    pub image_width: i32,
    pub image_height: i32,
    pub number_of_planes: u16,
    pub bits_per_pixel: u16,
    #[br(count = 4, map = vec_u8_to_string)]
    pub compression_type: String,
    pub image_size_in_bytes: i32,
    pub x_pels_per_meter: i32,
    pub y_pels_per_meter: i32,
    pub colors_used: i32,
    pub colors_important: i32,
}

#[binread]
#[derive(Debug, PartialEq, Clone)]
pub struct IndexEntry {
    #[br(temp)]
    buf: [u8; 4],
    #[br(calc = (buf[0] - b'0') * 10 + (buf[1] - b'0'))]
    pub stream_num: u8,
    #[br(calc = IndexEntryType::from(&[buf[2], buf[3]]))]
    pub kind: IndexEntryType,
    pub dw_flags: i32,
    pub dw_chunk_offset: i32,
    pub dw_chunk_length: i32,
}

#[binread]
#[derive(Debug, PartialEq, Clone)]
#[br(repr = u16)]
pub enum IndexEntryType {
    None,
    UncompressedVideoFrame,
    CompressedVideoFrame,
    PaletteChange = 4,
    AudioData = 8,
    VideoData = 3,
}

impl From<&[u8; 2]> for IndexEntryType {
    fn from(value: &[u8; 2]) -> Self {
        match value {
            b"db" => Self::UncompressedVideoFrame,
            b"dc" => Self::CompressedVideoFrame,
            b"pc" => Self::PaletteChange,
            b"wb" => Self::AudioData,
            _ => Self::None,
        }
    }
}

impl Avi {
    fn index_entries(&self) -> Option<&Vec<IndexEntry>> {
        self.chunks.iter().find_map(|chunk| match &chunk.kind {
            ChunkType::Idx1 { index_entries } => Some(index_entries),
            _ => None,
        })
    }

    fn movi_offset(&self) -> u64 {
        self.chunks
            .iter()
            .find_map(|chunk| {
                if chunk.kind == ChunkType::Movi {
                    Some(chunk.offset.pos)
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }

    pub fn avi_header(&self) -> Option<&AviHeader> {
        self.chunks.iter().find_map(|chunk| match &chunk.kind {
            ChunkType::Hdrl { avi_header, .. } => Some(avi_header),
            _ => None,
        })
    }

    pub fn stream_format_vid(&self) -> Option<&StreamFormat> {
        self.chunks.iter().find_map(|chunk| match &chunk.kind {
            ChunkType::Hdrl { chunks, .. } => chunks.iter().find_map(|chunk| {
                chunk.header.iter().find_map(|info_chunk| match info_chunk {
                    InfoChunk::StreamFormat(stream_format) => Some(stream_format),
                    _ => None,
                })
            }),
            _ => None,
        })
    }
}

pub fn read_avi(data: &[u8]) -> binrw::BinResult<(Avi, Vec<Frame>)> {
    let mut reader = Cursor::new(data);
    let avi: Avi = reader.read_le()?;
    let movi_offset = avi.movi_offset();

    let frames = match avi.index_entries() {
        Some(entries) => entries.iter().map(|entry| read_frame(&mut reader, entry, movi_offset)).collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };

    Ok((avi, frames))
}

fn read_frame<R>(
    reader: &mut R,
    index_entry: &IndexEntry,
    movi_offset: u64,
) -> binrw::BinResult<Frame>
where
    R: Read + Seek,
{
    reader.read_le_args((index_entry, movi_offset))
}
