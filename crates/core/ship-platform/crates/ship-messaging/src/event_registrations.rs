//! Event registrations for the MESSAGING domain.
//!
//! The legacy `human.response` registration has been removed (theme-A
//! contain A8): the webhook ingress never went live, no inbound emitter
//! ever published the event, and the gateway contract is a messaging
//! fast-follow. The outbound notifier still lives in
//! `ship-messaging::HumanGatewayService`; it is the seed the inbound
//! half will plug into when the fast-follow lands.
//!
//! MESSAGING's own event registrations will return here when the
//! inbound path lights up.
