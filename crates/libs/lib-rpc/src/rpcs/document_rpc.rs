use crate::config::rpc_config;
use crate::params::{ParamsForCreate, ParamsForUpdate, ParamsIded, ParamsList};
use crate::File;
use crate::Result;
use aws_sdk_s3::{primitives::ByteStream, Client};
use lib_core::ctx::Ctx;
use lib_core::model::document::{
	Document, DocumentBmc, DocumentFilter, DocumentForCreate, DocumentForRequest,
	DocumentForUpdate,
};
use lib_core::model::ModelManager;

pub async fn create_document(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsForCreate<DocumentForRequest>,
	file: File,
) -> Result<Document> {
	let s3_client = mm.bucket.clone();
	upload_to_s3(&s3_client, &file).await?;

	let ParamsForCreate { data } = params;

	let final_data = DocumentForCreate {
		archive_id: data.archive_id,
		name: file.file_name.clone(),
		doc_type: file.content_type.clone(),
		url: file.url.clone(),
	};

	let document_id = DocumentBmc::create(&ctx, &mm, final_data).await?;
	let document = DocumentBmc::get(&ctx, &mm, document_id).await?;

	Ok(document)
}

pub async fn list_documents(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsList<DocumentFilter>,
) -> Result<Vec<Document>> {
	let documents =
		DocumentBmc::list(&ctx, &mm, params.filters, params.list_options).await?;

	Ok(documents)
}

pub async fn get_document(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsIded,
) -> Result<Document> {
	let ParamsIded { id } = params;

	let document = DocumentBmc::get(&ctx, &mm, id).await?;

	Ok(document)
}

pub async fn update_document(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsForUpdate<DocumentForRequest>,
	file: Option<File>,
) -> Result<Document> {
	let s3_client = mm.bucket.clone();
	let ParamsForUpdate { id, data } = params;

	// Retrieve the updated document
	let document = DocumentBmc::get(&ctx, &mm, id).await?;

	let mut new_data = DocumentForUpdate {
		archive_id: data.archive_id,
		name: document.name,
		doc_type: document.doc_type,
		url: document.url,
	};

	if let Some(mut file) = file {
		// Upload the file to S3
		upload_to_s3(&s3_client, &mut file).await?;

		// Update the data with the file information
		new_data.name = file.file_name.clone();
		new_data.doc_type = file.content_type.clone();
		new_data.url = file.url.clone();
	}

	// Proceed to update the document
	DocumentBmc::update(&ctx, &mm, id, new_data).await?;

	// Retrieve the updated document
	let document = DocumentBmc::get(&ctx, &mm, id).await?;

	Ok(document)
}

pub async fn delete_document(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsIded,
) -> Result<Document> {
	let ParamsIded { id } = params;

	let document = DocumentBmc::get(&ctx, &mm, id).await?;
	DocumentBmc::delete(&ctx, &mm, id).await?;

	Ok(document)
}

async fn upload_to_s3(s3_client: &Client, file: &File) -> Result<()> {
	let res = s3_client
		.put_object()
		.bucket(&rpc_config().AWS_BUCKET_NAME)
		.content_type(file.content_type.clone())
		.content_length(file.bytes.len() as i64)
		.key(file.key.clone())
		.body(ByteStream::from(file.bytes.to_vec()))
		.send()
		.await;

	let _ = res.is_ok();

	Ok(())
}
