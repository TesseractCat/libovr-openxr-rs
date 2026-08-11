//! Resolver stubs for pnsovr.dll's platform dependency.

use core::ffi::{c_char, c_void};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

const MESSAGE_PLATFORM_INITIALIZED: u32 = 0x6da7_ba8f;
const MESSAGE_USER_GET_LOGGED_IN_USER: u32 = 0x436f_345d;
const MESSAGE_USER_GET_LOGGED_IN_USER_FRIENDS: u32 = 0x587c_2a8d;
const MESSAGE_USER_GET_ORG_SCOPED_ID: u32 = 0x18f0_b01b;
const MESSAGE_USER_GET_ACCESS_TOKEN: u32 = 0x06a8_5abe;
const MESSAGE_IAP_GET_VIEWER_PURCHASES: u32 = 0x3a0f_8419;
const MESSAGE_RICH_PRESENCE_GET_DESTINATIONS: u32 = 0x586f_2d14;
const MESSAGE_IAP_GET_PRODUCTS_BY_SKU: u32 = 0x7e9a_caf5;
const MESSAGE_USER_GET_USER_PROOF: u32 = 0x2281_0483;
const MESSAGE_NOTIFICATION_GET_ROOM_INVITES: u32 = 0x6f91_6b92;
const MESSAGE_ROOM_CREATE_AND_JOIN_PRIVATE2: u32 = 0x5a3a_6243;
const MESSAGE_ROOM_UPDATE_DATA_STORE: u32 = 0x026e_4028;
static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

#[repr(C)]
struct PlatformUser {
    id: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct KeyValuePair {
    key: *const c_char,
    value_type: i32,
    string_value: *const c_char,
    int_value: i32,
    double_value: f64,
}
#[repr(C)]
struct PlatformMessage {
    message_type: u32,
    request_id: u64,
    user: usize,
}

static EMPTY_ARRAY: u8 = 0;
static MESSAGE_QUEUE: Mutex<Vec<Box<PlatformMessage>>> = Mutex::new(Vec::new());
static BOOTSTRAP_MESSAGE_SENT: AtomicBool = AtomicBool::new(false);

fn local_user() -> &'static PlatformUser {
    static USER: std::sync::OnceLock<PlatformUser> = std::sync::OnceLock::new();
    USER.get_or_init(|| PlatformUser {
        id: crate::config::user_identity().id,
    })
}

fn local_org() -> &'static PlatformUser {
    static ORG: std::sync::OnceLock<PlatformUser> = std::sync::OnceLock::new();
    ORG.get_or_init(|| PlatformUser {
        id: crate::config::user_identity().org_id,
    })
}

fn queue_message(message_type: u32, user: usize) -> u64 {
    let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
    if let Ok(mut queue) = MESSAGE_QUEUE.lock() {
        queue.push(Box::new(PlatformMessage {
            message_type,
            request_id,
            user,
        }));
    }
    request_id
}

fn queue_logged_in_user() -> u64 {
    queue_message(
        MESSAGE_USER_GET_LOGGED_IN_USER,
        (local_user() as *const PlatformUser) as usize,
    )
}

fn queue_platform_initialized() -> u64 {
    queue_message(MESSAGE_PLATFORM_INITIALIZED, 0)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresence_GetNextDestinationArrayPage() {
    crate::capi::log_call("ovr_RichPresence_GetNextDestinationArrayPage");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresence_GetDestinations() -> u64 {
    crate::capi::log_call("ovr_RichPresence_GetDestinations");
    queue_message(MESSAGE_RICH_PRESENCE_GET_DESTINATIONS, 0)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresence_Clear() {
    crate::capi::log_call("ovr_RichPresence_Clear");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresenceOptions_SetStartTime() {
    crate::capi::log_call("ovr_RichPresenceOptions_SetStartTime");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresenceOptions_SetMaxCapacity() {
    crate::capi::log_call("ovr_RichPresenceOptions_SetMaxCapacity");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresenceOptions_SetIsJoinable() {
    crate::capi::log_call("ovr_RichPresenceOptions_SetIsJoinable");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresenceOptions_SetInstanceId() {
    crate::capi::log_call("ovr_RichPresenceOptions_SetInstanceId");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresenceOptions_SetExtraContext() {
    crate::capi::log_call("ovr_RichPresenceOptions_SetExtraContext");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresenceOptions_SetEndTime() {
    crate::capi::log_call("ovr_RichPresenceOptions_SetEndTime");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresenceOptions_SetDeeplinkMessageOverride() {
    crate::capi::log_call("ovr_RichPresenceOptions_SetDeeplinkMessageOverride");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresenceOptions_SetCurrentCapacity() {
    crate::capi::log_call("ovr_RichPresenceOptions_SetCurrentCapacity");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RoomOptions_Destroy() {
    crate::capi::log_call("ovr_RoomOptions_Destroy");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Net_SendPacket() {
    crate::capi::log_call("ovr_Net_SendPacket");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RoomOptions_SetOrdering() {
    crate::capi::log_call("ovr_RoomOptions_SetOrdering");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_CreateAndJoinPrivate2(
    _join_policy: i32,
    _max_users: u32,
    _room_options: *const c_void,
) -> u64 {
    crate::capi::log_call("ovr_Room_CreateAndJoinPrivate2");
    queue_message(MESSAGE_ROOM_CREATE_AND_JOIN_PRIVATE2, 0)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_Get() {
    crate::capi::log_call("ovr_Room_Get");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_GetInvitableUsers2() {
    crate::capi::log_call("ovr_Room_GetInvitableUsers2");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_InviteUser() {
    crate::capi::log_call("ovr_Room_InviteUser");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresence_Set() {
    crate::capi::log_call("ovr_RichPresence_Set");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_KickUser() {
    crate::capi::log_call("ovr_Room_KickUser");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_LaunchInvitableUserFlow() {
    crate::capi::log_call("ovr_Room_LaunchInvitableUserFlow");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_Leave() {
    crate::capi::log_call("ovr_Room_Leave");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_UpdateDataStore(
    _room_id: u64,
    _data: *const KeyValuePair,
    _num_items: u32,
) -> u64 {
    crate::capi::log_call("ovr_Room_UpdateDataStore");
    queue_message(MESSAGE_ROOM_UPDATE_DATA_STORE, 0)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_UpdateMembershipLockStatus() {
    crate::capi::log_call("ovr_Room_UpdateMembershipLockStatus");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_UpdateOwner() {
    crate::capi::log_call("ovr_Room_UpdateOwner");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_UpdatePrivateRoomJoinPolicy() {
    crate::capi::log_call("ovr_Room_UpdatePrivateRoomJoinPolicy");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetAccessToken() -> u64 {
    crate::capi::log_call("ovr_User_GetAccessToken");
    queue_message(MESSAGE_USER_GET_ACCESS_TOKEN, 0)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetLoggedInUser() -> u64 {
    crate::capi::log_call("ovr_User_GetLoggedInUser");
    queue_logged_in_user()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetLoggedInUserFriends() -> u64 {
    crate::capi::log_call("ovr_User_GetLoggedInUserFriends");
    queue_message(MESSAGE_USER_GET_LOGGED_IN_USER_FRIENDS, 0)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetLoggedInUserRecentlyMetUsersAndRooms() {
    crate::capi::log_call("ovr_User_GetLoggedInUserRecentlyMetUsersAndRooms");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetNextUserAndRoomArrayPage() {
    crate::capi::log_call("ovr_User_GetNextUserAndRoomArrayPage");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetNextUserArrayPage() {
    crate::capi::log_call("ovr_User_GetNextUserArrayPage");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetOrgScopedID(_user_id: u64) -> u64 {
    crate::capi::log_call("ovr_User_GetOrgScopedID");
    queue_message(MESSAGE_USER_GET_ORG_SCOPED_ID, 0)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetUserProof() -> u64 {
    crate::capi::log_call("ovr_User_GetUserProof");
    queue_message(MESSAGE_USER_GET_USER_PROOF, 0)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_ApplicationLifecycle_GetLaunchDetails() {
    crate::capi::log_call("ovr_ApplicationLifecycle_GetLaunchDetails");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Net_AcceptForCurrentRoom() {
    crate::capi::log_call("ovr_Net_AcceptForCurrentRoom");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Net_CloseForCurrentRoom() {
    crate::capi::log_call("ovr_Net_CloseForCurrentRoom");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Net_ReadPacket() {
    crate::capi::log_call("ovr_Net_ReadPacket");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_Join2() {
    crate::capi::log_call("ovr_Room_Join2");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RoomOptions_Create() {
    crate::capi::log_call("ovr_RoomOptions_Create");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresenceOptions_SetApiName() {
    crate::capi::log_call("ovr_RichPresenceOptions_SetApiName");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresenceOptions_Destroy() {
    crate::capi::log_call("ovr_RichPresenceOptions_Destroy");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresenceOptions_Create() {
    crate::capi::log_call("ovr_RichPresenceOptions_Create");
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_FreeMessage(message: *mut c_void) {
    crate::capi::log_call("ovr_FreeMessage");
    if !message.is_null() {
        unsafe { drop(Box::from_raw(message.cast::<PlatformMessage>())) };
    }
}

/// Pop one completion from the local offline message queue.
#[unsafe(no_mangle)]
pub extern "system" fn ovr_PopMessage() -> *mut c_void {
    crate::capi::log_call("ovr_PopMessage");
    // pnsovr may initialize internally before resolving the public Ex export.
    // Seed the documented init completion followed by the local-user result.
    if !BOOTSTRAP_MESSAGE_SENT.swap(true, Ordering::AcqRel) {
        queue_platform_initialized();
        queue_logged_in_user();
    }
    let message = MESSAGE_QUEUE
        .lock()
        .ok()
        .and_then(|mut queue| (!queue.is_empty()).then(|| queue.remove(0)));
    match message {
        Some(message) => {
            let message_type = message.message_type;
            let raw = Box::into_raw(message).cast::<c_void>();
            crate::capi::log_call(&format!(
                "ovr_PopMessage delivered type={message_type:#x} handle={raw:p}"
            ));
            raw
        }
        None => core::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetLoggedInUserID() -> u64 {
    crate::capi::log_call("ovr_GetLoggedInUserID");
    // Echo's offline path still requires a non-zero local principal. Zero
    // produces its "???-0" player records and later a null indirect call.
    local_user().id
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_IsPlatformInitialized() -> u8 {
    crate::capi::log_call("ovr_IsPlatformInitialized");
    1
}

// pnsovr resolves these initialization functions dynamically rather than
// importing them. Report success for the local/offline shim.
#[unsafe(no_mangle)]
pub extern "system" fn ovr_PlatformInitializeWindows(_app_id: *const c_char) -> i32 {
    crate::capi::log_call("ovr_PlatformInitializeWindows");
    0 // ovrPlatformInitialize_Success
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_PlatformInitializeWindowsAsynchronousEx(
    _app_id: *const c_char,
    out_result: *mut i32,
    _product_version: i32,
    _major_version: i32,
) -> u64 {
    crate::capi::log_call("ovr_PlatformInitializeWindowsAsynchronousEx");
    if !out_result.is_null() {
        unsafe { *out_result = 0 };
    }
    queue_platform_initialized()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_PlatformInitializeWindowsAsynchronous(_app_id: *const c_char) -> u64 {
    crate::capi::log_call("ovr_PlatformInitializeWindowsAsynchronous");
    queue_platform_initialized()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_PlatformInitializeStandaloneAccessToken() -> i32 {
    crate::capi::log_call("ovr_PlatformInitializeStandaloneAccessToken");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Platform_InitializeStandaloneOculus() -> i32 {
    crate::capi::log_call("ovr_Platform_InitializeStandaloneOculus");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_PlatformInitializeWithAccessToken() -> i32 {
    crate::capi::log_call("ovr_PlatformInitializeWithAccessToken");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_PlatformInitializeWithAccessTokenAndOptions() -> i32 {
    crate::capi::log_call("ovr_PlatformInitializeWithAccessTokenAndOptions");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Voip_Stop() {
    crate::capi::log_call("ovr_Voip_Stop");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Voip_Start() {
    crate::capi::log_call("ovr_Voip_Start");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Voip_SetMicrophoneMuted() {
    crate::capi::log_call("ovr_Voip_SetMicrophoneMuted");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Voip_GetPCMSize() {
    crate::capi::log_call("ovr_Voip_GetPCMSize");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Voip_GetPCM() {
    crate::capi::log_call("ovr_Voip_GetPCM");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Voip_GetOutputBufferMaxSize() -> usize {
    crate::capi::log_call("ovr_Voip_GetOutputBufferMaxSize");
    // This uses `size_t`. A void stub leaves RAX undefined,
    // which makes pnsovr treat an arbitrary value as an audio-buffer size.
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Voip_Accept() {
    crate::capi::log_call("ovr_Voip_Accept");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Net_SendPacketToCurrentRoom() {
    crate::capi::log_call("ovr_Net_SendPacketToCurrentRoom");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Notification_MarkAsRead() {
    crate::capi::log_call("ovr_Notification_MarkAsRead");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Notification_GetRoomInvites() -> u64 {
    crate::capi::log_call("ovr_Notification_GetRoomInvites");
    queue_message(MESSAGE_NOTIFICATION_GET_ROOM_INVITES, 0)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Notification_GetNextRoomInviteNotificationArrayPage() {
    crate::capi::log_call("ovr_Notification_GetNextRoomInviteNotificationArrayPage");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_IAP_LaunchCheckoutFlow() {
    crate::capi::log_call("ovr_IAP_LaunchCheckoutFlow");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_IAP_GetViewerPurchasesDurableCache() {
    crate::capi::log_call("ovr_IAP_GetViewerPurchasesDurableCache");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_IAP_GetViewerPurchases() -> u64 {
    crate::capi::log_call("ovr_IAP_GetViewerPurchases");
    queue_message(MESSAGE_IAP_GET_VIEWER_PURCHASES, 0)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_IAP_GetProductsBySKU(_skus: *const *const c_char, _count: i32) -> u64 {
    crate::capi::log_call("ovr_IAP_GetProductsBySKU");
    queue_message(MESSAGE_IAP_GET_PRODUCTS_BY_SKU, 0)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_IAP_GetNextPurchaseArrayPage() {
    crate::capi::log_call("ovr_IAP_GetNextPurchaseArrayPage");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_IAP_GetNextProductArrayPage() {
    crate::capi::log_call("ovr_IAP_GetNextProductArrayPage");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Entitlement_GetIsViewerEntitled() {
    crate::capi::log_call("ovr_Entitlement_GetIsViewerEntitled");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Packet_GetSize() {
    crate::capi::log_call("ovr_Packet_GetSize");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Packet_GetSenderID() {
    crate::capi::log_call("ovr_Packet_GetSenderID");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Packet_GetBytes() {
    crate::capi::log_call("ovr_Packet_GetBytes");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Packet_Free() {
    crate::capi::log_call("ovr_Packet_Free");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_IsError(_message: *const c_void) -> u8 {
    crate::capi::log_call("ovr_Message_IsError");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetUserProof(_obj: *const c_void) -> *const c_void {
    crate::capi::log_call("ovr_Message_GetUserProof");
    (&EMPTY_ARRAY as *const u8).cast()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetUserArray(_obj: *const c_void) -> *const c_void {
    crate::capi::log_call("ovr_Message_GetUserArray");
    (&EMPTY_ARRAY as *const u8).cast()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_VoipEncoder_AddPCM() {
    crate::capi::log_call("ovr_VoipEncoder_AddPCM");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_VoipEncoder_GetCompressedData() {
    crate::capi::log_call("ovr_VoipEncoder_GetCompressedData");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_VoipDecoder_Decode() {
    crate::capi::log_call("ovr_VoipDecoder_Decode");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_VoipDecoder_GetDecodedPCM() {
    crate::capi::log_call("ovr_VoipDecoder_GetDecodedPCM");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Microphone_GetPCM() {
    crate::capi::log_call("ovr_Microphone_GetPCM");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Microphone_Start() {
    crate::capi::log_call("ovr_Microphone_Start");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Microphone_Stop() {
    crate::capi::log_call("ovr_Microphone_Stop");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Voip_CreateEncoder() {
    crate::capi::log_call("ovr_Voip_CreateEncoder");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Voip_DestroyEncoder() {
    crate::capi::log_call("ovr_Voip_DestroyEncoder");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Voip_CreateDecoder() {
    crate::capi::log_call("ovr_Voip_CreateDecoder");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Voip_DestroyDecoder() {
    crate::capi::log_call("ovr_Voip_DestroyDecoder");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Microphone_Create() {
    crate::capi::log_call("ovr_Microphone_Create");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Microphone_Destroy() {
    crate::capi::log_call("ovr_Microphone_Destroy");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Destination_GetApiName(_obj: *const c_void) -> *const c_char {
    crate::capi::log_call("ovr_Destination_GetApiName");
    c"".as_ptr()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Destination_GetDisplayName(_obj: *const c_void) -> *const c_char {
    crate::capi::log_call("ovr_Destination_GetDisplayName");
    c"".as_ptr()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetInviteToken() {
    crate::capi::log_call("ovr_User_GetInviteToken");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetPresence() {
    crate::capi::log_call("ovr_User_GetPresence");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetPresenceDeeplinkMessage() {
    crate::capi::log_call("ovr_User_GetPresenceDeeplinkMessage");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetPresenceStatus() {
    crate::capi::log_call("ovr_User_GetPresenceStatus");
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_User_GetID(user: *const c_void) -> u64 {
    crate::capi::log_call("ovr_User_GetID");
    unsafe { user.cast::<PlatformUser>().as_ref() }.map_or(0, |user| user.id)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetOculusID(_user: *const c_void) -> *const c_char {
    crate::capi::log_call("ovr_User_GetOculusID");
    crate::config::user_identity().oculus_id.as_ptr()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_UserArray_GetElement(
    _obj: *const c_void,
    _index: usize,
) -> *const c_void {
    crate::capi::log_call("ovr_UserArray_GetElement");
    core::ptr::null()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_UserArray_GetSize(_obj: *const c_void) -> usize {
    crate::capi::log_call("ovr_UserArray_GetSize");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_UserArray_HasNextPage() {
    crate::capi::log_call("ovr_UserArray_HasNextPage");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_DataStore_GetValue() {
    crate::capi::log_call("ovr_DataStore_GetValue");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_DestinationArray_GetElement(
    _obj: *const c_void,
    _index: usize,
) -> *const c_void {
    crate::capi::log_call("ovr_DestinationArray_GetElement");
    core::ptr::null()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_DestinationArray_GetSize(_obj: *const c_void) -> usize {
    crate::capi::log_call("ovr_DestinationArray_GetSize");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_DestinationArray_HasNextPage(_obj: *const c_void) -> bool {
    crate::capi::log_call("ovr_DestinationArray_HasNextPage");
    false
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Error_GetMessage() {
    crate::capi::log_call("ovr_Error_GetMessage");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Error_GetCode() {
    crate::capi::log_call("ovr_Error_GetCode");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Error_GetHttpCode() {
    crate::capi::log_call("ovr_Error_GetHttpCode");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_LaunchDetails_GetDeeplinkMessage() {
    crate::capi::log_call("ovr_LaunchDetails_GetDeeplinkMessage");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_LaunchDetails_GetDestinationApiName() {
    crate::capi::log_call("ovr_LaunchDetails_GetDestinationApiName");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_LaunchDetails_GetLaunchSource() {
    crate::capi::log_call("ovr_LaunchDetails_GetLaunchSource");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_LaunchDetails_GetRoomID() {
    crate::capi::log_call("ovr_LaunchDetails_GetRoomID");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_LaunchDetails_GetLaunchType() {
    crate::capi::log_call("ovr_LaunchDetails_GetLaunchType");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_GetDataStore() {
    crate::capi::log_call("ovr_Room_GetDataStore");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_GetID(_obj: *const c_void) -> u64 {
    crate::capi::log_call("ovr_Room_GetID");
    local_user().id
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_GetIsMembershipLocked(_obj: *const c_void) -> bool {
    crate::capi::log_call("ovr_Room_GetIsMembershipLocked");
    false
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_GetJoinPolicy(_obj: *const c_void) -> i32 {
    crate::capi::log_call("ovr_Room_GetJoinPolicy");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_GetOwner(_obj: *const c_void) -> *const c_void {
    crate::capi::log_call("ovr_Room_GetOwner");
    (local_user() as *const PlatformUser).cast()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_GetUsers(_obj: *const c_void) -> *const c_void {
    crate::capi::log_call("ovr_Room_GetUsers");
    (&EMPTY_ARRAY as *const u8).cast()
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_OrgScopedID_GetID(obj: *const c_void) -> u64 {
    crate::capi::log_call("ovr_OrgScopedID_GetID");
    unsafe { obj.cast::<PlatformUser>().as_ref() }.map_or(0, |org| org.id)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Product_GetDescription() {
    crate::capi::log_call("ovr_Product_GetDescription");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Product_GetFormattedPrice() {
    crate::capi::log_call("ovr_Product_GetFormattedPrice");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Product_GetName() {
    crate::capi::log_call("ovr_Product_GetName");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Product_GetSKU() {
    crate::capi::log_call("ovr_Product_GetSKU");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_ProductArray_GetElement(
    _obj: *const c_void,
    _index: usize,
) -> *const c_void {
    crate::capi::log_call("ovr_ProductArray_GetElement");
    core::ptr::null()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_ProductArray_GetSize(_obj: *const c_void) -> usize {
    crate::capi::log_call("ovr_ProductArray_GetSize");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_ProductArray_HasNextPage(_obj: *const c_void) -> bool {
    crate::capi::log_call("ovr_ProductArray_HasNextPage");
    false
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Purchase_GetSKU(_obj: *const c_void) -> *const c_char {
    crate::capi::log_call("ovr_Purchase_GetSKU");
    c"".as_ptr()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_PurchaseArray_GetElement(
    _obj: *const c_void,
    _index: usize,
) -> *const c_void {
    crate::capi::log_call("ovr_PurchaseArray_GetElement");
    core::ptr::null()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_PurchaseArray_GetSize(_obj: *const c_void) -> usize {
    crate::capi::log_call("ovr_PurchaseArray_GetSize");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_PurchaseArray_HasNextPage(_obj: *const c_void) -> bool {
    crate::capi::log_call("ovr_PurchaseArray_HasNextPage");
    false
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RoomInviteNotification_GetID() {
    crate::capi::log_call("ovr_RoomInviteNotification_GetID");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RoomInviteNotification_GetRoomID() {
    crate::capi::log_call("ovr_RoomInviteNotification_GetRoomID");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RoomInviteNotification_GetSentTime() {
    crate::capi::log_call("ovr_RoomInviteNotification_GetSentTime");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RoomInviteNotificationArray_GetElement(
    _obj: *const c_void,
    _index: usize,
) -> *const c_void {
    crate::capi::log_call("ovr_RoomInviteNotificationArray_GetElement");
    core::ptr::null()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RoomInviteNotificationArray_GetSize(_obj: *const c_void) -> usize {
    crate::capi::log_call("ovr_RoomInviteNotificationArray_GetSize");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RoomInviteNotificationArray_HasNextPage(_obj: *const c_void) -> bool {
    crate::capi::log_call("ovr_RoomInviteNotificationArray_HasNextPage");
    false
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_UserAndRoom_GetUser() {
    crate::capi::log_call("ovr_UserAndRoom_GetUser");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_UserAndRoomArray_GetElement() {
    crate::capi::log_call("ovr_UserAndRoomArray_GetElement");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_UserAndRoomArray_GetSize() {
    crate::capi::log_call("ovr_UserAndRoomArray_GetSize");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_UserAndRoomArray_HasNextPage() {
    crate::capi::log_call("ovr_UserAndRoomArray_HasNextPage");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_UserProof_GetNonce(_obj: *const c_void) -> *const c_char {
    crate::capi::log_call("ovr_UserProof_GetNonce");
    c"local-user-proof".as_ptr()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetRoom(_obj: *const c_void) -> *const c_void {
    crate::capi::log_call("ovr_Message_GetRoom");
    (local_user() as *const PlatformUser).cast()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetRoomInviteNotification() {
    crate::capi::log_call("ovr_Message_GetRoomInviteNotification");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetRoomInviteNotificationArray(
    _obj: *const c_void,
) -> *const c_void {
    crate::capi::log_call("ovr_Message_GetRoomInviteNotificationArray");
    (&EMPTY_ARRAY as *const u8).cast()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetUserAndRoomArray() {
    crate::capi::log_call("ovr_Message_GetUserAndRoomArray");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetDestinationArray(_obj: *const c_void) -> *const c_void {
    crate::capi::log_call("ovr_Message_GetDestinationArray");
    (&EMPTY_ARRAY as *const u8).cast()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetError() {
    crate::capi::log_call("ovr_Message_GetError");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetOrgScopedID(_obj: *const c_void) -> *const c_void {
    crate::capi::log_call("ovr_Message_GetOrgScopedID");
    (local_org() as *const PlatformUser).cast()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetProductArray(_obj: *const c_void) -> *const c_void {
    crate::capi::log_call("ovr_Message_GetProductArray");
    (&EMPTY_ARRAY as *const u8).cast()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetPurchase() {
    crate::capi::log_call("ovr_Message_GetPurchase");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetPurchaseArray(_obj: *const c_void) -> *const c_void {
    crate::capi::log_call("ovr_Message_GetPurchaseArray");
    (&EMPTY_ARRAY as *const u8).cast()
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_Message_GetRequestID(message: *const c_void) -> u64 {
    crate::capi::log_call("ovr_Message_GetRequestID");
    unsafe { message.cast::<PlatformMessage>().as_ref() }.map_or(0, |message| message.request_id)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetString(_obj: *const c_void) -> *const c_char {
    crate::capi::log_call("ovr_Message_GetString");
    c"local-access-token".as_ptr()
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_Message_GetType(message: *const c_void) -> u32 {
    crate::capi::log_call("ovr_Message_GetType");
    unsafe { message.cast::<PlatformMessage>().as_ref() }.map_or(0, |message| message.message_type)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_Message_GetUser(message: *const c_void) -> *mut c_void {
    crate::capi::log_call("ovr_Message_GetUser");
    unsafe { message.cast::<PlatformMessage>().as_ref() }
        .map_or(core::ptr::null_mut(), |message| message.user as *mut c_void)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovrKeyValuePair_makeString(
    key: *const c_char,
    value: *const c_char,
) -> KeyValuePair {
    crate::capi::log_call("ovrKeyValuePair_makeString");
    KeyValuePair {
        key,
        value_type: 0, // ovrKeyValuePairType_String
        string_value: value,
        int_value: 0,
        double_value: 0.0,
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn ovrID_FromString() {
    crate::capi::log_call("ovrID_FromString");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovrLaunchType_ToString(_value: i32) -> *const c_char {
    crate::capi::log_call("ovrLaunchType_ToString");
    c"Unknown".as_ptr()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovrRoomJoinPolicy_ToString(_value: i32) -> *const c_char {
    crate::capi::log_call("ovrRoomJoinPolicy_ToString");
    c"Unknown".as_ptr()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovrPlatformInitializeResult_ToString(
    result: i32,
) -> *const core::ffi::c_char {
    crate::capi::log_call(&format!(
        "ovrPlatformInitializeResult_ToString result={result}"
    ));
    match result {
        0 => c"Success".as_ptr(),
        -1 => c"Uninitialized".as_ptr(),
        -2 => c"PreLoaded".as_ptr(),
        -3 => c"FileInvalid".as_ptr(),
        -4 => c"SignatureInvalid".as_ptr(),
        -5 => c"UnableToVerify".as_ptr(),
        -6 => c"VersionMismatch".as_ptr(),
        _ => c"Unknown".as_ptr(),
    }
}
