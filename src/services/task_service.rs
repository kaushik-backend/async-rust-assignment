use crate::{
    db::TaskDb,
    models::task_model::{CreateTask, Task},
};
use chrono::{DateTime as ChronoDateTime, NaiveDate};
use futures_util::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, DateTime};

fn parse_due_date(due_date: &Option<String>) -> Option<DateTime> {
    if let Some(d) = due_date {
        if let Ok(parsed) = ChronoDateTime::parse_from_rfc3339(d) {
            return Some(DateTime::from_millis(parsed.timestamp_millis()));
        }
        if let Ok(parsed) = NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            let dt = parsed.and_hms_opt(0, 0, 0).unwrap().and_utc();
            return Some(DateTime::from_millis(dt.timestamp_millis()));
        }
    }
    None
}

fn now() -> DateTime {
    DateTime::now()
}

pub async fn create_task(db: &TaskDb, payload: CreateTask, created_by: ObjectId) -> Task {
    let new_task = Task {
        id: None,
        title: payload.title,
        description: payload.description,
        status: payload.status.unwrap_or_else(|| "todo".into()),
        priority: payload.priority.unwrap_or_else(|| "medium".into()),
        assignee: payload.assignee,
        due_date: parse_due_date(&payload.due_date),
        created_by,
        created_at: now(),
        updated_at: now(),
    };

    let collection = db.lock().await;
    let result = collection.insert_one(&new_task, None).await.unwrap();

    let mut created_task = new_task.clone();
    created_task.id = result.inserted_id.as_object_id();

    created_task
}

pub async fn get_all_tasks(db: &TaskDb, user_id: Option<&str>) -> Vec<Task> {
    let collection = db.lock().await;

    let filter = if let Some(uid) = user_id {
        if let Ok(user_obj_id) = ObjectId::parse_str(uid) {
            doc! { "created_by": user_obj_id }
        } else {
            doc! {}
        }
    } else {
        doc! {}
    };

    let mut cursor = collection.find(filter, None).await.unwrap();
    let mut tasks = vec![];

    while let Some(task) = cursor.try_next().await.unwrap() {
        tasks.push(task);
    }

    tasks
}

pub async fn get_task_by_id(db: &TaskDb, id: &str, user_id: Option<&str>) -> Option<Task> {
    let obj_id = ObjectId::parse_str(id).ok()?;
    let collection = db.lock().await;

    let filter = if let Some(uid) = user_id {
        if let Ok(user_obj_id) = ObjectId::parse_str(uid) {
            doc! { "_id": obj_id, "created_by": user_obj_id }
        } else {
            doc! { "_id": obj_id }
        }
    } else {
        doc! { "_id": obj_id }
    };

    collection.find_one(filter, None).await.unwrap()
}

pub async fn update_task(
    db: &TaskDb,
    id: &str,
    payload: CreateTask,
    user_id: &str,
) -> Option<Task> {
    let obj_id = ObjectId::parse_str(id).unwrap();
    let user_obj_id = ObjectId::parse_str(user_id).unwrap();
    let collection = db.lock().await;

    // Only update if owned by user
    collection
        .update_one(
            doc! { "_id": obj_id, "created_by": user_obj_id },
            doc! {
                "$set": {
                    "title": payload.title,
                    "description": payload.description,
                    "status": payload.status.unwrap_or_else(|| "todo".into()),
                    "priority": payload.priority.unwrap_or_else(|| "medium".into()),
                    "assignee": payload.assignee,
                    "due_date": parse_due_date(&payload.due_date),
                    "updated_at": now(),
                }
            },
            None,
        )
        .await
        .unwrap();

    collection
        .find_one(doc! { "_id": obj_id, "created_by": user_obj_id }, None)
        .await
        .unwrap()
}

pub async fn delete_task(db: &TaskDb, id: &str, user_id: &str) -> bool {
    let obj_id = ObjectId::parse_str(id).unwrap();
    let user_obj_id = ObjectId::parse_str(user_id).unwrap();
    let collection = db.lock().await;
    collection
        .delete_one(doc! { "_id": obj_id, "created_by": user_obj_id }, None)
        .await
        .unwrap()
        .deleted_count
        > 0
}
