#![warn(missing_docs)]

/*!
 # Message pickup

 This is the implementation of the message pickup protocol v3.0.
 It is used to facilitate an agent picking up messages held at a mediator.
 */

mod web;
mod model;
mod repository;
mod constants;
pub mod error;
pub mod plugin;
