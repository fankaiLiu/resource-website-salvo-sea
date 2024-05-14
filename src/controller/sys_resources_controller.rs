use crate::{
    app_writer::{AppWriter, ErrorResponseBuilder},
    dtos::sys_resources_dto::{
        PaginationParams, SysResourceChangeLink, SysResourceCreateRequest, SysResourceList,
        SysResourceResponse,
    },
    middleware::jwt,
    services::sys_resource_service,
    utils::app_error::AppError,
};
use salvo::{
    http::StatusCode,
    oapi::{
        endpoint,
        extract::{JsonBody, PathParam, QueryParam},
    },
    prelude::Json,
    Depot, Request, Response, Writer,
};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[endpoint(tags("删除页面截图"))]
pub async fn delete_image(req: PathParam<String>, depot: &mut Depot) -> AppWriter<()> {
    let token = depot.get::<&str>("jwt_token").copied().unwrap();

    if let Err(err) = jwt::parse_token(token) {
        return AppWriter(Err(err.into()));
    }
    let jwt_model = jwt::parse_token(token).unwrap();
    let uuid = jwt_model.user_id;
    let image_uuid = req.0;
    let result = sys_resource_service::delete_image(image_uuid, uuid).await;
    AppWriter(result)
}

#[endpoint(tags("根据uuid获取资源详情"))]
pub async fn get_resource_detail_by_uuid(
    req: &mut Request,
    depot: &mut Depot,
) -> AppWriter<SysResourceResponse> {
    // 从url获取uuid
    let resource_uuid = req.query::<String>("resource").unwrap();

    let token = depot.get::<&str>("jwt_token").copied().unwrap();

    if let Err(err) = jwt::parse_token(token) {
        return AppWriter(Err(err.into()));
    }
    let jwt_model = jwt::parse_token(token).unwrap();
    let uuid = jwt_model.user_id;
    let role: Option<u32> = jwt_model.role;

    let result = sys_resource_service::get_resource_detail_by_uuid(resource_uuid, uuid, role).await;
    AppWriter(result)
}

#[endpoint(tags("根据类型和语言获取资源列表"))]
pub async fn get_resources_of_category_and_language(
    // req: &mut Request,
    query: PaginationParams,
) -> AppWriter<Vec<SysResourceList>> {
    // 从路径中获取language和category
    let category = query.0;
    let language = query.1;
    let page = query.2;
    let page_size = query.3;
    // let language = query.param::<String>("language").unwrap();
    // let category = query.param::<String>("category").unwrap();

    // 从查询参数中获取分页参数
    // let page = query.param::<u64>("page").unwrap_or(1);
    // let page_size = query.param::<u64>("pageSize").unwrap_or(49);
    // 调用service处理
    match sys_resource_service::get_resources_by_category_and_language(
        category, language, page, page_size,
    )
    .await
    {
        Ok(result) => AppWriter(Ok(result)),
        Err(err) => AppWriter(Err(err)),
    }
}

#[endpoint(tags("根据类型获取资源列表"))]
pub async fn get_resources_of_category(req: &mut Request) -> AppWriter<Vec<SysResourceList>> {
    // 从url中获取category
    let category = req.query::<String>("category").unwrap();

    // 从请求中获取分页参数
    let page = req.query::<u64>("page").unwrap_or(1);
    let page_size = req.query::<u64>("page_size").unwrap_or(49);
    // 调用service处理
    match sys_resource_service::get_resources_of_category(category, page, page_size).await {
        Ok(result) => AppWriter(Ok(result)),
        Err(err) => AppWriter(Err(err)),
    }
}

#[endpoint(tags("根据语言获取资源列表"))]
pub async fn get_resource_list_of_language(
    language: QueryParam<String, false>,
    req: &mut Request,
) -> AppWriter<Vec<SysResourceList>> {
    // 从url中获取language和category
    // let language = req.query::<String>("language").unwrap();
    let language = language.as_deref().unwrap_or("PHP");

    // 从请求中获取分页参数
    let page = req.query::<u64>("page").unwrap_or(1);
    let page_size = req.query::<u64>("page_size").unwrap_or(49);
    // 调用service处理
    match sys_resource_service::get_resours_of_language(language.to_string(), page, page_size).await
    {
        Ok(result) => AppWriter(Ok(result)),
        Err(err) => AppWriter(Err(err)),
    }
}

#[endpoint(tags("获取资源列表"))]
pub async fn get_resource_list(req: &mut Request) -> AppWriter<Vec<SysResourceList>> {
    // 从请求中获取查询条件
    let _query = req.query::<String>("all").unwrap_or("".to_string());
    // 从请求中获取分页参数
    let page = req.query::<u64>("page").unwrap_or(1);
    let page_size = req.query::<u64>("page_size").unwrap_or(49);
    // 调用service处理
    match sys_resource_service::get_resource_list(page, page_size).await {
        Ok(result) => AppWriter(Ok(result)),
        Err(err) => AppWriter(Err(err)),
    }
}

#[endpoint(tags("更改下载链接"))]
pub async fn put_change_link(form_data: JsonBody<SysResourceChangeLink>, res: &mut Response) {
    let cloned_form_data = form_data.0;
    let resource_link = cloned_form_data.resource_link.clone();
    if let Err(_err) = sys_resource_service::change_resource_link(cloned_form_data).await {
        res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
    }
    res.render(Json(format!("改资源的下载链接已更改为{}", resource_link)));
}

#[endpoint(tags("新建源码包"))]
pub async fn post_create_resource(
    form_data: JsonBody<SysResourceCreateRequest>,
    depot: &mut Depot,
    res: &mut Response,
) {
    let form_data = form_data.0;

    let token = depot.get::<&str>("jwt_token").copied().unwrap();

    if let Err(err) = jwt::parse_token(token) {
        return ErrorResponseBuilder::with_err(AppError::AnyHow(err)).into_response(res);
    }

    let jwt_model = jwt::parse_token(token).unwrap();
    let uuid = jwt_model.user_id;

    if let Err(_err) = sys_resource_service::create_resource(form_data, uuid).await {
        res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
    }
    res.status_code(StatusCode::CREATED);
}

#[endpoint(tags("上传描述文件"))]
pub async fn put_upload_description(req: &mut Request, res: &mut Response) {
    let file = req.file("description").await;
    if let Some(file) = file {
        let mime = file.content_type().unwrap().to_string();
        if mime.starts_with("text/") {
            let file_name = Uuid::new_v4().to_string();
            let mut dest = PathBuf::from("../assets/uploads/description/");

            // 提取原始文件名和扩展名
            let original_file_name = file.name().unwrap_or("file");
            let extension = match Path::new(original_file_name).extension() {
                Some(extension) => extension.to_string_lossy().to_lowercase(),
                None => return,
            };
            // 判断上传的描述文件类型是否为.md或.txt
            if !extension.eq("md") || !extension.eq("txt") {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json("文件类型错误，请上传.md或.txt文件"));
            }

            // 构建新的文件名（保留原始文件的扩展名）
            dest.push(format!("{}.{}", file_name, extension));

            // 保存文件
            let info = if let Err(e) = std::fs::copy(file.path(), &dest) {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                format!("file not found in request: {}", e)
            } else {
                res.status_code(StatusCode::OK);
                format!("{:?}", dest)
            };

            res.render(Json(info));
        }
    } else {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json("file not found in request"));
    }
}

//上传资源截图
#[endpoint(tags("上传图片"))]
pub async fn put_upload_image(req: &mut Request, res: &mut Response) {
    let files = req.files("avatar").await;
    if let Some(files) = files {
        let mut msgs: Vec<(String, String)> = Vec::with_capacity(files.len());
        for file in files {
            let mime = file.content_type().unwrap().to_string();
            if mime.starts_with("image/") {
                let file_name = Uuid::new_v4().to_string();
                let mut dest = PathBuf::from("../assets/uploads/avatar/");

                // 提取原始文件名和扩展名
                let original_file_name = file.name().unwrap_or("file");
                let extension = Path::new(original_file_name)
                    .extension()
                    .unwrap_or_default();

                // 构建新的文件名（保留原始文件的扩展名）
                dest.push(format!(
                    "{}.{}",
                    file_name,
                    extension.to_str().unwrap_or("jpg")
                ));

                if let Err(e) = std::fs::copy(file.path(), &dest) {
                    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                    res.render(Json(format!("file not found in request: {}", e)));
                } else {
                    msgs.push((dest.to_string_lossy().to_string(), file_name.clone()));
                }
            }
        }
        let _resulr = sys_resource_service::save_resource_image(msgs.clone());
        if let Err(e) = _resulr.await {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(format!("file not found in request: {}", e)));
        };
        res.render(Json(&msgs));
    } else {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Json("file not found in request"));
    }
}
