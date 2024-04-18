use std::io;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive as _, ToPrimitive as _};
use thiserror::Error;

use crate::cursor::{ReadCursor, WriteCursor};
use crate::{decode, utils, PduDecode, PduEncode, PduError, PduResult};

mod bitmap;
mod bitmap_cache;
mod bitmap_codecs;
mod brush;
mod frame_acknowledge;
mod general;
mod glyph_cache;
mod input;
mod large_pointer;
mod multifragment_update;
mod offscreen_bitmap_cache;
mod order;
mod pointer;
mod sound;
mod surface_commands;
mod virtual_channel;

pub use self::bitmap::{Bitmap, BitmapDrawingFlags};
pub use self::bitmap_cache::{
    BitmapCache, BitmapCacheRev2, CacheEntry, CacheFlags, CellInfo, BITMAP_CACHE_ENTRIES_NUM,
};
pub use self::bitmap_codecs::{
    BitmapCodecs, CaptureFlags, Codec, CodecProperty, EntropyBits, Guid, NsCodec, RemoteFxContainer, RfxCaps,
    RfxCapset, RfxClientCapsContainer, RfxICap, RfxICapFlags,
};
pub use self::brush::{Brush, SupportLevel};
pub use self::color_cache::ColorCache;
pub use self::control::Control;
pub use self::font::{Font, FontSupportFlags};
pub use self::frame_acknowledge::FrameAcknowledge;
pub use self::general::{General, GeneralExtraFlags, MajorPlatformType, MinorPlatformType, PROTOCOL_VER};
pub use self::glyph_cache::{CacheDefinition, GlyphCache, GlyphSupportLevel, GLYPH_CACHE_NUM};
pub use self::input::{Input, InputFlags};
pub use self::large_pointer::{LargePointer, LargePointerSupportFlags};
pub use self::multifragment_update::MultifragmentUpdate;
pub use self::offscreen_bitmap_cache::OffscreenBitmapCache;
pub use self::order::{Order, OrderFlags, OrderSupportExFlags, OrderSupportIndex};
pub use self::pointer::Pointer;
pub use self::sound::{Sound, SoundFlags};
pub use self::surface_commands::{CmdFlags, SurfaceCommands};
pub use self::virtual_channel::{VirtualChannel, VirtualChannelFlags};

pub const SERVER_CHANNEL_ID: u16 = 0x03ea;

const SOURCE_DESCRIPTOR_LENGTH_FIELD_SIZE: usize = 2;
const COMBINED_CAPABILITIES_LENGTH_FIELD_SIZE: usize = 2;
const NUMBER_CAPABILITIES_FIELD_SIZE: usize = 2;
const PADDING_SIZE: usize = 2;
const SESSION_ID_FIELD_SIZE: usize = 4;
const CAPABILITY_SET_TYPE_FIELD_SIZE: usize = 2;
const CAPABILITY_SET_LENGTH_FIELD_SIZE: usize = 2;
const ORIGINATOR_ID_FIELD_SIZE: usize = 2;

const NULL_TERMINATOR: &str = "\0";

/// [2.2.1.13.1] Server Demand Active PDU
///
/// [2.2.1.13.1]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/a07abad1-38bb-4a1a-96c9-253e3d5440df
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerDemandActive {
    pub pdu: DemandActive,
}

impl ServerDemandActive {
    const NAME: &'static str = "ServerDemandActive";

    const FIXED_PART_SIZE: usize = SESSION_ID_FIELD_SIZE;
}

impl PduEncode for ServerDemandActive {
    fn encode(&self, dst: &mut WriteCursor<'_>) -> PduResult<()> {
        ensure_size!(in: dst, size: self.size());

        self.pdu.encode(dst)?;
        dst.write_u32(0); // This field is ignored by the client

        Ok(())
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn size(&self) -> usize {
        Self::FIXED_PART_SIZE + self.pdu.size()
    }
}

impl<'de> PduDecode<'de> for ServerDemandActive {
    fn decode(src: &mut ReadCursor<'de>) -> PduResult<Self> {
        let pdu = DemandActive::decode(src)?;

        ensure_size!(in: src, size: 4);
        let _session_id = src.read_u32();

        Ok(Self { pdu })
    }
}

/// [2.2.1.13.2] Client Confirm Active PDU
///
/// [2.2.1.13.2]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/4c3c2710-0bf0-4c54-8e69-aff40ffcde66
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientConfirmActive {
    /// According to [MSDN](https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/4e9722c3-ad83-43f5-af5a-529f73d88b48),
    /// this field MUST be set to [SERVER_CHANNEL_ID](constant.SERVER_CHANNEL_ID.html).
    /// However, the Microsoft RDP client takes this value from a server's
    /// [PduSource](https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/73d01865-2eae-407f-9b2c-87e31daac471)
    /// field of the [Server Demand Active PDU](https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/bd612af5-cb54-43a2-9646-438bc3ecf5db).
    /// Therefore, checking the `originator_id` field is the responsibility of the user of the library.
    pub originator_id: u16,
    pub pdu: DemandActive,
}

impl ClientConfirmActive {
    const NAME: &'static str = "ClientConfirmActive";

    const FIXED_PART_SIZE: usize = ORIGINATOR_ID_FIELD_SIZE;
}

impl PduEncode for ClientConfirmActive {
    fn encode(&self, dst: &mut WriteCursor<'_>) -> PduResult<()> {
        ensure_fixed_part_size!(in: dst);

        dst.write_u16(self.originator_id);

        self.pdu.encode(dst)
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn size(&self) -> usize {
        Self::FIXED_PART_SIZE + self.pdu.size()
    }
}

impl<'de> PduDecode<'de> for ClientConfirmActive {
    fn decode(src: &mut ReadCursor<'de>) -> PduResult<Self> {
        ensure_fixed_part_size!(in: src);

        let originator_id = src.read_u16();
        let pdu = DemandActive::decode(src)?;

        Ok(Self { originator_id, pdu })
    }
}

/// 2.2.1.13.1.1 Demand Active PDU Data (TS_DEMAND_ACTIVE_PDU)
///
/// [2.2.1.13.1.1]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/bd612af5-cb54-43a2-9646-438bc3ecf5db
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemandActive {
    pub source_descriptor: String,
    pub capability_sets: Vec<CapabilitySet>,
}

impl DemandActive {
    const NAME: &'static str = "DemandActive";

    const FIXED_PART_SIZE: usize = SOURCE_DESCRIPTOR_LENGTH_FIELD_SIZE + COMBINED_CAPABILITIES_LENGTH_FIELD_SIZE;
}

impl PduEncode for DemandActive {
    fn encode(&self, dst: &mut WriteCursor<'_>) -> PduResult<()> {
        ensure_size!(in: dst, size: self.size());

        let combined_length = self.capability_sets.iter().map(PduEncode::size).sum::<usize>()
            + NUMBER_CAPABILITIES_FIELD_SIZE
            + PADDING_SIZE;

        dst.write_u16(cast_length!(
            "sourceDescLen",
            self.source_descriptor.len() + NULL_TERMINATOR.as_bytes().len()
        )?);
        dst.write_u16(cast_length!("combinedLen", combined_length)?);
        dst.write_slice(self.source_descriptor.as_ref());
        dst.write_slice(NULL_TERMINATOR.as_bytes());
        dst.write_u16(cast_length!("len", self.capability_sets.len())?);
        write_padding!(dst, 2);

        for capability_set in self.capability_sets.iter() {
            capability_set.encode(dst)?;
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn size(&self) -> usize {
        Self::FIXED_PART_SIZE
            + self.source_descriptor.len()
            + 1
            + NUMBER_CAPABILITIES_FIELD_SIZE
            + PADDING_SIZE
            + self.capability_sets.iter().map(PduEncode::size).sum::<usize>()
    }
}

impl<'de> PduDecode<'de> for DemandActive {
    fn decode(src: &mut ReadCursor<'de>) -> PduResult<Self> {
        ensure_fixed_part_size!(in: src);

        let source_descriptor_length = src.read_u16() as usize;
        // The combined size in bytes of the numberCapabilities, pad2Octets, and capabilitySets fields.
        let _combined_capabilities_length = src.read_u16() as usize;

        ensure_size!(in: src, size: source_descriptor_length);
        let source_descriptor = utils::decode_string(
            src.read_slice(source_descriptor_length),
            utils::CharacterSet::Ansi,
            false,
        )?;

        ensure_size!(in: src, size: 2 + 2);
        let capability_sets_count = src.read_u16() as usize;
        let _padding = src.read_u16();

        let mut capability_sets = Vec::with_capacity(capability_sets_count);
        for _ in 0..capability_sets_count {
            capability_sets.push(CapabilitySet::decode(src)?);
        }

        Ok(Self {
            source_descriptor,
            capability_sets,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilitySet {
    // mandatory
    General(General),
    Bitmap(Bitmap),
    Order(Order),
    BitmapCache(BitmapCache),
    BitmapCacheRev2(BitmapCacheRev2),
    Pointer(Pointer),
    Sound(Sound),
    Input(Input),
    Brush(Brush),
    GlyphCache(GlyphCache),
    OffscreenBitmapCache(OffscreenBitmapCache),
    VirtualChannel(VirtualChannel),

    // optional
    Control(Control),
    WindowActivation(Vec<u8>),
    Share(Vec<u8>),
    Font(Font),
    BitmapCacheHostSupport(Vec<u8>),
    DesktopComposition(Vec<u8>),
    MultiFragmentUpdate(MultifragmentUpdate),
    LargePointer(LargePointer),
    SurfaceCommands(SurfaceCommands),
    BitmapCodecs(BitmapCodecs),

    // other
    ColorCache(ColorCache),
    DrawNineGridCache(Vec<u8>),
    DrawGdiPlus(Vec<u8>),
    Rail(Vec<u8>),
    WindowList(Vec<u8>),
    FrameAcknowledge(FrameAcknowledge),
}

impl CapabilitySet {
    const NAME: &'static str = "CapabilitySet";

    const FIXED_PART_SIZE: usize = CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE;
}

impl PduEncode for CapabilitySet {
    fn encode(&self, dst: &mut WriteCursor<'_>) -> PduResult<()> {
        ensure_size!(in: dst, size: self.size());

        match self {
            CapabilitySet::General(capset) => {
                dst.write_u16(CapabilitySetType::General.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::Bitmap(capset) => {
                dst.write_u16(CapabilitySetType::Bitmap.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::Order(capset) => {
                dst.write_u16(CapabilitySetType::Order.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::BitmapCache(capset) => {
                dst.write_u16(CapabilitySetType::BitmapCache.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::BitmapCacheRev2(capset) => {
                dst.write_u16(CapabilitySetType::BitmapCacheRev2.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::Pointer(capset) => {
                dst.write_u16(CapabilitySetType::Pointer.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::Sound(capset) => {
                dst.write_u16(CapabilitySetType::Sound.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::Input(capset) => {
                dst.write_u16(CapabilitySetType::Input.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::Brush(capset) => {
                dst.write_u16(CapabilitySetType::Brush.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::GlyphCache(capset) => {
                dst.write_u16(CapabilitySetType::GlyphCache.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::OffscreenBitmapCache(capset) => {
                dst.write_u16(CapabilitySetType::OffscreenBitmapCache.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::VirtualChannel(capset) => {
                dst.write_u16(CapabilitySetType::VirtualChannel.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::SurfaceCommands(capset) => {
                dst.write_u16(CapabilitySetType::SurfaceCommands.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::BitmapCodecs(capset) => {
                dst.write_u16(CapabilitySetType::BitmapCodecs.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::MultiFragmentUpdate(capset) => {
                dst.write_u16(CapabilitySetType::MultiFragmentUpdate.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::LargePointer(capset) => {
                dst.write_u16(CapabilitySetType::LargePointer.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::FrameAcknowledge(capset) => {
                dst.write_u16(CapabilitySetType::FrameAcknowledge.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::Font(capset) => {
                dst.write_u16(CapabilitySetType::Font.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::Control(capset) => {
                dst.write_u16(CapabilitySetType::Control.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            CapabilitySet::ColorCache(capset) => {
                dst.write_u16(CapabilitySetType::ColorCache.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capset.size() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                capset.encode(dst)?;
            }
            _ => {
                let (capability_set_type, capability_set_buffer) = match self {
                    CapabilitySet::WindowActivation(buffer) => (CapabilitySetType::WindowActivation, buffer),
                    CapabilitySet::Share(buffer) => (CapabilitySetType::Share, buffer),
                    CapabilitySet::BitmapCacheHostSupport(buffer) => {
                        (CapabilitySetType::BitmapCacheHostSupport, buffer)
                    }
                    CapabilitySet::DesktopComposition(buffer) => (CapabilitySetType::DesktopComposition, buffer),
                    CapabilitySet::DrawNineGridCache(buffer) => (CapabilitySetType::DrawNineGridCache, buffer),
                    CapabilitySet::DrawGdiPlus(buffer) => (CapabilitySetType::DrawGdiPlus, buffer),
                    CapabilitySet::Rail(buffer) => (CapabilitySetType::Rail, buffer),
                    CapabilitySet::WindowList(buffer) => (CapabilitySetType::WindowList, buffer),
                    _ => unreachable!(),
                };

                dst.write_u16(capability_set_type.to_u16().unwrap());
                dst.write_u16(cast_length!(
                    "len",
                    capability_set_buffer.len() + CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE
                )?);
                dst.write_slice(capability_set_buffer);
            }
        };
        Ok(())
    }

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn size(&self) -> usize {
        Self::FIXED_PART_SIZE
            + match self {
                CapabilitySet::General(capset) => capset.size(),
                CapabilitySet::Bitmap(capset) => capset.size(),
                CapabilitySet::Order(capset) => capset.size(),
                CapabilitySet::BitmapCache(capset) => capset.size(),
                CapabilitySet::BitmapCacheRev2(capset) => capset.size(),
                CapabilitySet::Pointer(capset) => capset.size(),
                CapabilitySet::Sound(capset) => capset.size(),
                CapabilitySet::Input(capset) => capset.size(),
                CapabilitySet::Brush(capset) => capset.size(),
                CapabilitySet::GlyphCache(capset) => capset.size(),
                CapabilitySet::OffscreenBitmapCache(capset) => capset.size(),
                CapabilitySet::VirtualChannel(capset) => capset.size(),
                CapabilitySet::SurfaceCommands(capset) => capset.size(),
                CapabilitySet::BitmapCodecs(capset) => capset.size(),
                CapabilitySet::MultiFragmentUpdate(capset) => capset.size(),
                CapabilitySet::LargePointer(capset) => capset.size(),
                CapabilitySet::FrameAcknowledge(capset) => capset.size(),
                CapabilitySet::Font(capset) => capset.size(),
                CapabilitySet::Control(capset) => capset.size(),
                CapabilitySet::ColorCache(capset) => capset.size(),
                CapabilitySet::WindowActivation(buffer)
                | CapabilitySet::Share(buffer)
                | CapabilitySet::BitmapCacheHostSupport(buffer)
                | CapabilitySet::DesktopComposition(buffer)
                | CapabilitySet::DrawNineGridCache(buffer)
                | CapabilitySet::DrawGdiPlus(buffer)
                | CapabilitySet::Rail(buffer)
                | CapabilitySet::WindowList(buffer) => buffer.len(),
            }
    }
}

impl<'de> PduDecode<'de> for CapabilitySet {
    fn decode(src: &mut ReadCursor<'de>) -> PduResult<Self> {
        ensure_fixed_part_size!(in: src);

        let capability_set_type = CapabilitySetType::from_u16(src.read_u16())
            .ok_or_else(|| invalid_message_err!("capabilitySetType", "invalid capability set type"))?;

        let length = src.read_u16() as usize;

        if length < CAPABILITY_SET_TYPE_FIELD_SIZE + CAPABILITY_SET_LENGTH_FIELD_SIZE {
            return Err(invalid_message_err!("len", "invalid capability set length"));
        }

        let buffer_length = length - CAPABILITY_SET_TYPE_FIELD_SIZE - CAPABILITY_SET_LENGTH_FIELD_SIZE;
        ensure_size!(in: src, size: buffer_length);
        let capability_set_buffer = src.read_slice(buffer_length);

        match capability_set_type {
            CapabilitySetType::General => Ok(CapabilitySet::General(decode(capability_set_buffer)?)),
            CapabilitySetType::Bitmap => Ok(CapabilitySet::Bitmap(decode(capability_set_buffer)?)),
            CapabilitySetType::Order => Ok(CapabilitySet::Order(decode(capability_set_buffer)?)),
            CapabilitySetType::BitmapCache => Ok(CapabilitySet::BitmapCache(decode(capability_set_buffer)?)),
            CapabilitySetType::BitmapCacheRev2 => Ok(CapabilitySet::BitmapCacheRev2(decode(capability_set_buffer)?)),
            CapabilitySetType::Pointer => Ok(CapabilitySet::Pointer(decode(capability_set_buffer)?)),
            CapabilitySetType::Sound => Ok(CapabilitySet::Sound(decode(capability_set_buffer)?)),
            CapabilitySetType::Input => Ok(CapabilitySet::Input(decode(capability_set_buffer)?)),
            CapabilitySetType::Brush => Ok(CapabilitySet::Brush(decode(capability_set_buffer)?)),
            CapabilitySetType::GlyphCache => Ok(CapabilitySet::GlyphCache(decode(capability_set_buffer)?)),
            CapabilitySetType::OffscreenBitmapCache => {
                Ok(CapabilitySet::OffscreenBitmapCache(decode(capability_set_buffer)?))
            }
            CapabilitySetType::VirtualChannel => Ok(CapabilitySet::VirtualChannel(decode(capability_set_buffer)?)),
            CapabilitySetType::SurfaceCommands => Ok(CapabilitySet::SurfaceCommands(decode(capability_set_buffer)?)),
            CapabilitySetType::BitmapCodecs => Ok(CapabilitySet::BitmapCodecs(decode(capability_set_buffer)?)),
            CapabilitySetType::Font => Ok(CapabilitySet::Font(decode(capability_set_buffer)?)),
            CapabilitySetType::Control => Ok(CapabilitySet::Control(decode(capability_set_buffer)?)),
            CapabilitySetType::ColorCache => Ok(CapabilitySet::ColorCache(decode(capability_set_buffer)?)),
            CapabilitySetType::LargePointer => Ok(CapabilitySet::LargePointer(decode(capability_set_buffer)?)),
            CapabilitySetType::FrameAcknowledge => Ok(CapabilitySet::FrameAcknowledge(decode(capability_set_buffer)?)),

            CapabilitySetType::WindowActivation => Ok(CapabilitySet::WindowActivation(capability_set_buffer.into())),
            CapabilitySetType::Share => Ok(CapabilitySet::Share(capability_set_buffer.into())),
            CapabilitySetType::BitmapCacheHostSupport => {
                Ok(CapabilitySet::BitmapCacheHostSupport(capability_set_buffer.into()))
            }
            CapabilitySetType::DesktopComposition => {
                Ok(CapabilitySet::DesktopComposition(capability_set_buffer.into()))
            }
            CapabilitySetType::MultiFragmentUpdate => {
                Ok(CapabilitySet::MultiFragmentUpdate(decode(capability_set_buffer)?))
            }
            CapabilitySetType::DrawNineGridCache => Ok(CapabilitySet::DrawNineGridCache(capability_set_buffer.into())),
            CapabilitySetType::DrawGdiPlus => Ok(CapabilitySet::DrawGdiPlus(capability_set_buffer.into())),
            CapabilitySetType::Rail => Ok(CapabilitySet::Rail(capability_set_buffer.into())),
            CapabilitySetType::WindowList => Ok(CapabilitySet::WindowList(capability_set_buffer.into())),
        }
    }
}

#[derive(Copy, Clone, Debug, FromPrimitive, ToPrimitive)]
enum CapabilitySetType {
    General = 0x01,
    Bitmap = 0x02,
    Order = 0x03,
    BitmapCache = 0x04,
    Control = 0x05,
    WindowActivation = 0x07,
    Pointer = 0x08,
    Share = 0x09,
    ColorCache = 0x0a,
    Sound = 0x0c,
    Input = 0x0d,
    Font = 0x0e,
    Brush = 0x0f,
    GlyphCache = 0x10,
    OffscreenBitmapCache = 0x11,
    BitmapCacheHostSupport = 0x12,
    BitmapCacheRev2 = 0x13,
    VirtualChannel = 0x14,
    DrawNineGridCache = 0x15,
    DrawGdiPlus = 0x16,
    Rail = 0x17,
    WindowList = 0x18,
    DesktopComposition = 0x19,
    MultiFragmentUpdate = 0x1a,
    LargePointer = 0x1b,
    SurfaceCommands = 0x1c,
    BitmapCodecs = 0x1d,
    FrameAcknowledge = 0x1e,
}

#[derive(Debug, Error)]
pub enum CapabilitySetsError {
    #[error("IO error")]
    IOError(#[from] io::Error),
    #[error("UTF-8 error")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("invalid type field")]
    InvalidType,
    #[error("invalid bitmap compression field")]
    InvalidCompressionFlag,
    #[error("invalid multiple rectangle support field")]
    InvalidMultipleRectSupport,
    #[error("invalid protocol version field")]
    InvalidProtocolVersion,
    #[error("invalid compression types field")]
    InvalidCompressionTypes,
    #[error("invalid update capability flags field")]
    InvalidUpdateCapFlag,
    #[error("invalid remote unshare flag field")]
    InvalidRemoteUnshareFlag,
    #[error("invalid compression level field")]
    InvalidCompressionLevel,
    #[error("invalid brush support level field")]
    InvalidBrushSupportLevel,
    #[error("invalid glyph support level field")]
    InvalidGlyphSupportLevel,
    #[error("invalid RemoteFX capability version")]
    InvalidRfxICapVersion,
    #[error("invalid RemoteFX capability tile size")]
    InvalidRfxICapTileSize,
    #[error("invalid RemoteFXICap color conversion bits")]
    InvalidRfxICapColorConvBits,
    #[error("invalid RemoteFXICap transform bits")]
    InvalidRfxICapTransformBits,
    #[error("invalid RemoteFXICap entropy bits field")]
    InvalidRfxICapEntropyBits,
    #[error("invalid RemoteFX capability set block type")]
    InvalidRfxCapsetBlockType,
    #[error("invalid RemoteFX capability set type")]
    InvalidRfxCapsetType,
    #[error("invalid RemoteFX capabilities block type")]
    InvalidRfxCapsBlockType,
    #[error("invalid RemoteFX capabilities block length")]
    InvalidRfxCapsBockLength,
    #[error("invalid number of capability sets in RemoteFX capabilities")]
    InvalidRfxCapsNumCapsets,
    #[error("invalid codec property field")]
    InvalidCodecProperty,
    #[error("invalid codec ID")]
    InvalidCodecID,
    #[error("invalid channel chunk size field")]
    InvalidChunkSize,
    #[error("invalid codec property length for the current property ID")]
    InvalidPropertyLength,
    #[error("invalid data length")]
    InvalidLength,
    #[error("PDU error: {0}")]
    Pdu(PduError),
}

impl From<PduError> for CapabilitySetsError {
    fn from(e: PduError) -> Self {
        Self::Pdu(e)
    }
}

mod font {
    use crate::{
        cursor::{ReadCursor, WriteCursor},
        PduDecode, PduEncode, PduResult,
    };

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct FontSupportFlags(u16);

    impl FontSupportFlags {
        pub const FONTSUPPORT_FONTLIST: Self = Self(0x0001);
    }

    /// 2.2.7.2.5 Font Capability Set (TS_FONT_CAPABILITYSET)
    ///
    /// [2.2.7.2.5]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/18b4ccdc-e5b0-43c4-a453-cfa8c9feb2a4
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct Font {
        pub font_support_flags: FontSupportFlags,
    }

    impl Font {
        const FIXED_PART_SIZE: usize = 4;
        const NAME: &'static str = "Font";
    }

    impl PduEncode for Font {
        fn encode(&self, dst: &mut WriteCursor<'_>) -> PduResult<()> {
            ensure_fixed_part_size!(in: dst);
            dst.write_u16(self.font_support_flags.0);
            dst.write_u16(0); // pad2octets
            Ok(())
        }

        fn name(&self) -> &'static str {
            Self::NAME
        }

        fn size(&self) -> usize {
            Self::FIXED_PART_SIZE
        }
    }

    impl<'de> PduDecode<'de> for Font {
        fn decode(src: &mut ReadCursor<'de>) -> PduResult<Self> {
            ensure_fixed_part_size!(in: src);
            let font_support_flags = FontSupportFlags(src.read_u16());
            src.read_u16(); // pad2octets
            Ok(Self { font_support_flags })
        }
    }
}

mod control {
    use crate::{
        cursor::{ReadCursor, WriteCursor},
        PduDecode, PduEncode, PduResult,
    };

    /// 2.2.7.2.2 Control Capability Set (TS_CONTROL_CAPABILITYSET)
    ///
    /// [2.2.7.2.2]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/e0add8ac-1546-4091-85ba-0ea77f54f2c7
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct Control {
        control_flags: u16,
        remote_detach_flag: u16,
        control_interest: u16,
        detach_interest: u16,
    }

    impl Default for Control {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Control {
        const FIXED_PART_SIZE: usize = 8;
        const NAME: &'static str = "Control";

        pub fn new() -> Self {
            Self {
                control_flags: 0x0000,      // SHOULD be set to zero.
                remote_detach_flag: 0x0000, // SHOULD be set to FALSE (0x0000).
                control_interest: 0x0002,   // SHOULD be set to CONTROLPRIORITY_NEVER (0x0002).
                detach_interest: 0x0002,    // SHOULD be set to CONTROLPRIORITY_NEVER (0x0002).
            }
        }
    }

    impl PduEncode for Control {
        fn encode(&self, dst: &mut WriteCursor<'_>) -> PduResult<()> {
            ensure_fixed_part_size!(in: dst);
            dst.write_u16(self.control_flags);
            dst.write_u16(self.remote_detach_flag);
            dst.write_u16(self.control_interest);
            dst.write_u16(self.detach_interest);
            Ok(())
        }

        fn name(&self) -> &'static str {
            Self::NAME
        }

        fn size(&self) -> usize {
            Self::FIXED_PART_SIZE
        }
    }

    impl<'de> PduDecode<'de> for Control {
        fn decode(src: &mut ReadCursor<'de>) -> PduResult<Self> {
            ensure_fixed_part_size!(in: src);
            let control_flags = src.read_u16();
            let remote_detach_flag = src.read_u16();
            let control_interest = src.read_u16();
            let detach_interest = src.read_u16();
            Ok(Self {
                control_flags,
                remote_detach_flag,
                control_interest,
                detach_interest,
            })
        }
    }
}

mod color_cache {
    use crate::{
        cursor::{ReadCursor, WriteCursor},
        PduDecode, PduEncode, PduResult,
    };

    /// 2.2.1.1 Color Table Cache Capability Set (TS_COLORTABLE_CAPABILITYSET)
    ///
    /// [2.2.1.1]: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpegdi/2b7c6946-3612-4291-95a8-03b7b1387eaf
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct ColorCache {
        pub color_cache_table_size: u16,
    }

    impl Default for ColorCache {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ColorCache {
        const FIXED_PART_SIZE: usize = 4;
        const NAME: &'static str = "ColorCache";

        pub fn new() -> Self {
            Self {
                color_cache_table_size: 0x0006, // MUST be ignored during capability exchange and is assumed to be 0x0006.
            }
        }
    }

    impl PduEncode for ColorCache {
        fn encode(&self, dst: &mut WriteCursor<'_>) -> PduResult<()> {
            ensure_fixed_part_size!(in: dst);
            dst.write_u16(self.color_cache_table_size);
            dst.write_u16(0); // pad2octets
            Ok(())
        }

        fn name(&self) -> &'static str {
            Self::NAME
        }

        fn size(&self) -> usize {
            Self::FIXED_PART_SIZE
        }
    }

    impl<'de> PduDecode<'de> for ColorCache {
        fn decode(src: &mut ReadCursor<'de>) -> PduResult<Self> {
            ensure_fixed_part_size!(in: src);
            let color_cache_table_size = src.read_u16();
            src.read_u16(); // pad2octets
            Ok(Self { color_cache_table_size })
        }
    }
}
