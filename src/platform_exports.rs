//! Resolver stubs for pnsovr.dll's LibOVR Platform SDK dependency.

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresence_GetNextDestinationArrayPage() {
    crate::capi::log_call("ovr_RichPresence_GetNextDestinationArrayPage");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RichPresence_GetDestinations() -> u64 {
    crate::capi::log_call("ovr_RichPresence_GetDestinations");
    0
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
pub extern "system" fn ovr_Room_CreateAndJoinPrivate2() {
    crate::capi::log_call("ovr_Room_CreateAndJoinPrivate2");
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
pub extern "system" fn ovr_Room_UpdateDataStore() {
    crate::capi::log_call("ovr_Room_UpdateDataStore");
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
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetLoggedInUser() -> u64 {
    crate::capi::log_call("ovr_User_GetLoggedInUser");
    0 // invalid ovrRequest: do not create a request with no completion message
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetLoggedInUserFriends() {
    crate::capi::log_call("ovr_User_GetLoggedInUserFriends");
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
pub extern "system" fn ovr_User_GetOrgScopedID() -> u64 {
    crate::capi::log_call("ovr_User_GetOrgScopedID");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetUserProof() {
    crate::capi::log_call("ovr_User_GetUserProof");
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
pub extern "system" fn ovr_FreeMessage() {
    crate::capi::log_call("ovr_FreeMessage");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetLoggedInUserID() -> u64 {
    crate::capi::log_call("ovr_GetLoggedInUserID");
    0 // no Oculus account is available in the offline shim
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_IsPlatformInitialized() -> u8 {
    crate::capi::log_call("ovr_IsPlatformInitialized");
    1
}

// pnsovr resolves these initialization functions dynamically rather than
// importing them. Report success for the local/offline Platform SDK shim.
#[unsafe(no_mangle)]
pub extern "system" fn ovr_PlatformInitializeWindows() -> i32 {
    crate::capi::log_call("ovr_PlatformInitializeWindows");
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_PlatformInitializeWindowsAsynchronous() -> *mut core::ffi::c_void {
    crate::capi::log_call("ovr_PlatformInitializeWindowsAsynchronous");
    core::ptr::null_mut()
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
pub extern "system" fn ovr_Voip_GetOutputBufferMaxSize() {
    crate::capi::log_call("ovr_Voip_GetOutputBufferMaxSize");
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
    0
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
    0
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_IAP_GetProductsBySKU() {
    crate::capi::log_call("ovr_IAP_GetProductsBySKU");
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
pub extern "system" fn ovr_Message_IsError() {
    crate::capi::log_call("ovr_Message_IsError");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetUserProof() {
    crate::capi::log_call("ovr_Message_GetUserProof");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetUserArray() {
    crate::capi::log_call("ovr_Message_GetUserArray");
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
pub extern "system" fn ovr_Destination_GetApiName() {
    crate::capi::log_call("ovr_Destination_GetApiName");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Destination_GetDisplayName() {
    crate::capi::log_call("ovr_Destination_GetDisplayName");
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
pub extern "system" fn ovr_User_GetID() {
    crate::capi::log_call("ovr_User_GetID");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_User_GetOculusID() {
    crate::capi::log_call("ovr_User_GetOculusID");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_UserArray_GetElement() {
    crate::capi::log_call("ovr_UserArray_GetElement");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_UserArray_GetSize() {
    crate::capi::log_call("ovr_UserArray_GetSize");
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
pub extern "system" fn ovr_DestinationArray_GetElement() {
    crate::capi::log_call("ovr_DestinationArray_GetElement");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_DestinationArray_GetSize() {
    crate::capi::log_call("ovr_DestinationArray_GetSize");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_DestinationArray_HasNextPage() {
    crate::capi::log_call("ovr_DestinationArray_HasNextPage");
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
pub extern "system" fn ovr_Room_GetID() {
    crate::capi::log_call("ovr_Room_GetID");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_GetIsMembershipLocked() {
    crate::capi::log_call("ovr_Room_GetIsMembershipLocked");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_GetJoinPolicy() {
    crate::capi::log_call("ovr_Room_GetJoinPolicy");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_GetOwner() {
    crate::capi::log_call("ovr_Room_GetOwner");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Room_GetUsers() {
    crate::capi::log_call("ovr_Room_GetUsers");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_OrgScopedID_GetID() {
    crate::capi::log_call("ovr_OrgScopedID_GetID");
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
pub extern "system" fn ovr_ProductArray_GetElement() {
    crate::capi::log_call("ovr_ProductArray_GetElement");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_ProductArray_GetSize() {
    crate::capi::log_call("ovr_ProductArray_GetSize");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_ProductArray_HasNextPage() {
    crate::capi::log_call("ovr_ProductArray_HasNextPage");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Purchase_GetSKU() {
    crate::capi::log_call("ovr_Purchase_GetSKU");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_PurchaseArray_GetElement() {
    crate::capi::log_call("ovr_PurchaseArray_GetElement");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_PurchaseArray_GetSize() {
    crate::capi::log_call("ovr_PurchaseArray_GetSize");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_PurchaseArray_HasNextPage() {
    crate::capi::log_call("ovr_PurchaseArray_HasNextPage");
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
pub extern "system" fn ovr_RoomInviteNotificationArray_GetElement() {
    crate::capi::log_call("ovr_RoomInviteNotificationArray_GetElement");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RoomInviteNotificationArray_GetSize() {
    crate::capi::log_call("ovr_RoomInviteNotificationArray_GetSize");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RoomInviteNotificationArray_HasNextPage() {
    crate::capi::log_call("ovr_RoomInviteNotificationArray_HasNextPage");
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
pub extern "system" fn ovr_UserProof_GetNonce() {
    crate::capi::log_call("ovr_UserProof_GetNonce");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetRoom() {
    crate::capi::log_call("ovr_Message_GetRoom");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetRoomInviteNotification() {
    crate::capi::log_call("ovr_Message_GetRoomInviteNotification");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetRoomInviteNotificationArray() {
    crate::capi::log_call("ovr_Message_GetRoomInviteNotificationArray");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetUserAndRoomArray() {
    crate::capi::log_call("ovr_Message_GetUserAndRoomArray");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetDestinationArray() {
    crate::capi::log_call("ovr_Message_GetDestinationArray");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetError() {
    crate::capi::log_call("ovr_Message_GetError");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetOrgScopedID() {
    crate::capi::log_call("ovr_Message_GetOrgScopedID");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetProductArray() {
    crate::capi::log_call("ovr_Message_GetProductArray");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetPurchase() {
    crate::capi::log_call("ovr_Message_GetPurchase");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetPurchaseArray() {
    crate::capi::log_call("ovr_Message_GetPurchaseArray");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetRequestID() {
    crate::capi::log_call("ovr_Message_GetRequestID");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetString() {
    crate::capi::log_call("ovr_Message_GetString");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetType() {
    crate::capi::log_call("ovr_Message_GetType");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Message_GetUser() {
    crate::capi::log_call("ovr_Message_GetUser");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovrKeyValuePair_makeString() {
    crate::capi::log_call("ovrKeyValuePair_makeString");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovrID_FromString() {
    crate::capi::log_call("ovrID_FromString");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovrLaunchType_ToString() {
    crate::capi::log_call("ovrLaunchType_ToString");
}

#[unsafe(no_mangle)]
pub extern "system" fn ovrRoomJoinPolicy_ToString() {
    crate::capi::log_call("ovrRoomJoinPolicy_ToString");
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
