use anyhow::Context;
use color_print::cprintln;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use oauth2::ClientId;

use crate::net::microsoft::{
	self,
	auth::{self, mc_access_token_to_string},
	MinecraftUserProfile,
};

use super::{User, UserKind};

/// Present the login page and secret code to the user
pub fn present_login_page_and_code(url: &str, code: &str, o: &mut impl MCVMOutput) {
	let result = open::that_detached("url");
	if result.is_err() {
		o.display(
			MessageContents::Error("Failed to open link in browser".to_string()),
			MessageLevel::Important,
		);
	}

	o.display(
		MessageContents::Property(
			"Open this link in your web browser if it has not opened already".to_string(),
			Box::new(MessageContents::Hyperlink(url.to_string())),
		),
		MessageLevel::Important,
	);
	o.display(
		MessageContents::Property(
			"and enter the code".to_string(),
			Box::new(MessageContents::Copyable(code.to_string())),
		),
		MessageLevel::Important,
	);
}

impl User {
	/// Authenticate the user
	pub async fn authenticate(
		&mut self,
		client_id: ClientId,
		client: &reqwest::Client,
		o: &mut impl MCVMOutput,
	) -> anyhow::Result<()> {
		match &mut self.kind {
			UserKind::Microsoft { xbox_uid } => {
				let auth_result = authenticate_microsoft_user(client_id, &client, o)
					.await
					.context("Failed to authenticate user")?;
				let certificate =
					crate::net::microsoft::get_user_certificate(&auth_result.access_token, &client)
						.await
						.context("Failed to get user certificate")?;
				self.access_token = Some(auth_result.access_token);
				self.uuid = Some(auth_result.profile.uuid);
				self.keypair = Some(certificate.key_pair);
				*xbox_uid = Some(auth_result.xbox_uid);
			}
			UserKind::Demo | UserKind::Unverified => {}
		}

		Ok(())
	}
}

/// Result from the Microsoft authentication function
pub struct MicrosoftAuthResult {
	pub access_token: String,
	pub profile: MinecraftUserProfile,
	pub xbox_uid: String,
}

pub async fn authenticate_microsoft_user(
	client_id: ClientId,
	client: &reqwest::Client,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<MicrosoftAuthResult> {
	let oauth_client = auth::create_client(client_id).context("Failed to create OAuth client")?;
	let response = auth::generate_login_page(&oauth_client)
		.await
		.context("Failed to execute authorization and generate login page")?;

	present_login_page_and_code(
		response.verification_uri(),
		response.user_code().secret(),
		o,
	);

	let token = auth::get_microsoft_token(&oauth_client, response)
		.await
		.context("Failed to get Microsoft token")?;
	let mc_token = auth::auth_minecraft(token, reqwest::Client::new())
		.await
		.context("Failed to get Minecraft token")?;
	let access_token = mc_access_token_to_string(mc_token.access_token())?;

	let profile = microsoft::get_user_profile(&access_token, client)
		.await
		.context("Failed to get user profile")?;

	o.display(
		MessageContents::Success("Authentication successful".to_string()),
		MessageLevel::Important,
	);

	let out = MicrosoftAuthResult {
		access_token,
		profile,
		xbox_uid: mc_token.username().clone(),
	};

	Ok(out)
}

/// Authenticate with lots of prints; used for debugging
pub async fn debug_authenticate(
	client_id: ClientId,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<()> {
	cprintln!("<y>Note: This authentication is not complete and is for debug purposes only");
	println!("Client ID: {}", client_id.as_str());
	let client = auth::create_client(client_id).context("Failed to create OAuth client")?;
	let req_client = reqwest::Client::new();
	let response = auth::generate_login_page(&client)
		.await
		.context("Failed to execute authorization and generate login page")?;

	present_login_page_and_code(
		response.verification_uri(),
		response.user_code().secret(),
		o,
	);

	let token = auth::get_microsoft_token(&client, response)
		.await
		.context("Failed to get Microsoft token")?;

	cprintln!("Microsoft token: <b>{token:?}");

	let mc_token = auth::auth_minecraft(token, reqwest::Client::new())
		.await
		.context("Failed to get Minecraft token")?;

	cprintln!("Minecraft token: <b>{mc_token:?}");

	let access_token = mc_access_token_to_string(mc_token.access_token())?;
	cprintln!("Minecraft Access Token: <b>{access_token}");

	let profile = microsoft::get_user_profile(&access_token, &req_client)
		.await
		.context("Failed to get user profile")?;
	cprintln!("Profile: <b>{profile:?}");

	Ok(())
}
