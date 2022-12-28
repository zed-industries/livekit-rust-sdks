use crate::proto;
use crate::proto::ParticipantInfo;
use crate::room::id::{ParticipantIdentity, ParticipantSid, TrackSid};
use crate::room::participant::local_participant::LocalParticipant;
use crate::room::participant::remote_participant::RemoteParticipant;
use crate::room::publication::{TrackPublication, TrackPublicationTrait};
use crate::room::room_session::SessionEmitter;
use livekit_utils::enum_dispatch;
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::Arc;

pub mod local_participant;
pub mod remote_participant;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum ConnectionQuality {
    Unknown,
    Excellent,
    Good,
    Poor,
}

impl From<u8> for ConnectionQuality {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Excellent,
            2 => Self::Good,
            3 => Self::Poor,
            _ => Self::Unknown,
        }
    }
}

impl From<proto::ConnectionQuality> for ConnectionQuality {
    fn from(value: proto::ConnectionQuality) -> Self {
        match value {
            proto::ConnectionQuality::Excellent => Self::Excellent,
            proto::ConnectionQuality::Good => Self::Good,
            proto::ConnectionQuality::Poor => Self::Poor,
        }
    }
}

#[derive(Debug)]
pub(super) struct ParticipantShared {
    pub(super) sid: Mutex<ParticipantSid>,
    pub(super) identity: Mutex<ParticipantIdentity>,
    pub(super) name: Mutex<String>,
    pub(super) metadata: Mutex<String>,
    pub(super) tracks: RwLock<HashMap<TrackSid, TrackPublication>>,
    pub(super) speaking: AtomicBool,
    pub(super) audio_level: AtomicU32,
    pub(super) connection_quality: AtomicU8,
    pub(super) internal_tx: SessionEmitter,
}

impl ParticipantShared {
    pub(super) fn new(
        sid: ParticipantSid,
        identity: ParticipantIdentity,
        name: String,
        metadata: String,
        internal_tx: SessionEmitter,
    ) -> Self {
        Self {
            sid: Mutex::new(sid),
            identity: Mutex::new(identity),
            name: Mutex::new(name),
            metadata: Mutex::new(metadata),
            tracks: Default::default(),
            speaking: Default::default(),
            audio_level: Default::default(),
            connection_quality: AtomicU8::new(ConnectionQuality::Unknown as u8),
            internal_tx,
        }
    }

    pub(crate) fn update_info(&self, info: ParticipantInfo) {
        *self.sid.lock() = info.sid.into();
        *self.identity.lock() = info.identity.into();
        *self.name.lock() = info.name;
        *self.metadata.lock() = info.metadata; // TODO(theomonnom): callback MetadataChanged
    }

    pub(crate) fn set_speaking(&self, speaking: bool) {
        self.speaking.store(speaking, Ordering::SeqCst);
    }

    pub(crate) fn set_audio_level(&self, audio_level: f32) {
        self.audio_level
            .store(audio_level.to_bits(), Ordering::SeqCst)
    }

    pub(crate) fn set_connection_quality(&self, quality: ConnectionQuality) {
        self.connection_quality
            .store(quality as u8, Ordering::SeqCst);
    }

    pub(crate) fn add_track_publication(&self, publication: TrackPublication) {
        self.tracks.write().insert(publication.sid(), publication);
    }
}

pub(crate) trait ParticipantInternalTrait {
    fn set_speaking(&self, speaking: bool);
    fn set_audio_level(&self, level: f32);
    fn set_connection_quality(&self, quality: ConnectionQuality);
    fn update_info(self: &Arc<Self>, info: ParticipantInfo);
}

pub trait ParticipantTrait {
    fn sid(&self) -> ParticipantSid;
    fn identity(&self) -> ParticipantIdentity;
    fn name(&self) -> String;
    fn metadata(&self) -> String;
    fn is_speaking(&self) -> bool;
    fn audio_level(&self) -> f32;
    fn connection_quality(&self) -> ConnectionQuality;
}

#[derive(Debug, Clone)]
pub enum ParticipantHandle {
    Local(Arc<LocalParticipant>),
    Remote(Arc<RemoteParticipant>),
}

impl ParticipantHandle {
    enum_dispatch!(
        [Local, Remote]
        fnc!(pub(crate), update_info, &Self, [info: ParticipantInfo], ());
        fnc!(pub(crate), set_speaking, &Self, [speaking: bool], ());
        fnc!(pub(crate), set_audio_level, &Self, [audio_level: f32], ());
        fnc!(pub(crate), set_connection_quality, &Self, [quality: ConnectionQuality], ());
    );
}

impl ParticipantTrait for ParticipantHandle {
    enum_dispatch!(
        [Local, Remote]
        fnc!(sid, &Self, [], ParticipantSid);
        fnc!(identity, &Self, [], ParticipantIdentity);
        fnc!(name, &Self, [], String);
        fnc!(metadata, &Self, [], String);
        fnc!(is_speaking, &Self, [], bool);
        fnc!(audio_level, &Self, [], f32);
        fnc!(connection_quality, &Self, [], ConnectionQuality);
    );
}

macro_rules! impl_participant_trait {
    ($x:ty) => {
        use std::sync::atomic::Ordering;
        use std::sync::Arc;
        use $crate::proto::ParticipantInfo;
        use $crate::room::id::{ParticipantIdentity, ParticipantSid};
        use $crate::room::participant::ConnectionQuality;

        impl crate::room::participant::ParticipantTrait for $x {
            fn sid(&self) -> ParticipantSid {
                self.shared.sid.lock().clone()
            }

            fn identity(&self) -> ParticipantIdentity {
                self.shared.identity.lock().clone()
            }

            fn name(&self) -> String {
                self.shared.name.lock().clone()
            }

            fn metadata(&self) -> String {
                self.shared.metadata.lock().clone()
            }

            fn is_speaking(&self) -> bool {
                self.shared.speaking.load(Ordering::SeqCst)
            }

            fn audio_level(&self) -> f32 {
                f32::from_bits(self.shared.audio_level.load(Ordering::SeqCst))
            }

            fn connection_quality(&self) -> ConnectionQuality {
                self.shared.connection_quality.load(Ordering::SeqCst).into()
            }
        }
    };
}

pub(super) use impl_participant_trait;
