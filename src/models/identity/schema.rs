#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use chrono::NaiveDate;
use diesel::{deserialize::Queryable, table, Selectable};
use schemars::JsonSchema;
use serde::Serialize;
use chrono::offset::Utc;
use chrono::DateTime;
use diesel::Identifiable;
use diesel::Associations;


table! {
    django_admin_log (id) {
        action_time ->Timestamptz,
        user_id ->BigInt,
        content_type_id ->Nullable<BigInt>,
        object_id ->Nullable<Varchar>,
        #[max_length = 200]
        object_repr ->Varchar,
        action_flag ->BigInt,
        change_message ->Varchar,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable,Associations)]
#[diesel(belongs_to(Administrator, foreign_key = user_id))]#[diesel(belongs_to(ContentType, foreign_key = content_type_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = django_admin_log)]
pub struct LogEntry {
    pub action_time:DateTime<Utc>,
    pub user_id:i64,
    pub content_type_id:Option<i64>,
    pub object_id:Option<String>,
    pub object_repr:String,
    pub action_flag:i64,
    pub change_message:String,
    pub id:i64,
    
}

table! {
    auth_permission (id) {
        #[max_length = 255]
        name ->Varchar,
        content_type_id ->BigInt,
        #[max_length = 100]
        codename ->Varchar,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable,Associations)]
#[diesel(belongs_to(ContentType, foreign_key = content_type_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = auth_permission)]
pub struct Permission {
    pub name:String,
    pub content_type_id:i64,
    pub codename:String,
    pub id:i64,
    
}

table! {
    auth_group (id) {
        #[max_length = 150]
        name ->Varchar,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable)]

#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = auth_group)]
pub struct Group {
    pub name:String,
    pub id:i64,
    
}

table! {
    auth_group_permissions (id) {
        id ->Int8,
        permission_id ->Int8,
        group_id ->Int8,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable,Associations)]
#[diesel(belongs_to(Permission, foreign_key = permission_id))]#[diesel(belongs_to(Group, foreign_key = group_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = auth_group_permissions)]
pub struct Group_Permission {
    pub id:i64,
    pub permission_id:i64,
    pub group_id:i64,
    
}

table! {
    django_content_type (id) {
        #[max_length = 100]
        app_label ->Varchar,
        #[max_length = 100]
        model ->Varchar,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable)]

#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = django_content_type)]
pub struct ContentType {
    pub app_label:String,
    pub model:String,
    pub id:i64,
    
}

table! {
    django_session (session_key) {
        session_key ->Varchar,
        session_data ->Varchar,
        expire_date ->Timestamptz,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema)]

#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = django_session)]
pub struct Session {
    pub session_key:String,
    pub session_data:String,
    pub expire_date:DateTime<Utc>,
    
}

table! {
    tenant (id) {
        #[max_length = 200]
        name ->Varchar,
        is_active ->Bool,
        expiry_date ->Nullable<Date>,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable)]

#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = tenant)]
pub struct Tenant {
    pub name:String,
    pub is_active:bool,
    pub expiry_date:Option<NaiveDate>,
    pub id:i64,
    
}

table! {
    administrator (id) {
        #[max_length = 200]
        name ->Varchar,
        #[max_length = 20]
        mobile ->Nullable<Varchar>,
        #[max_length = 255]
        email ->Varchar,
        tenant_id ->Nullable<BigInt>,
        is_active ->Bool,
        is_staff ->Bool,
        is_admin ->Bool,
        password_change_required ->Bool,
        next_password_change_due ->Nullable<Date>,
        #[max_length = 128]
        password ->Varchar,
        last_login ->Nullable<Timestamptz>,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable,Associations)]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = administrator)]
pub struct Administrator {
    pub name:String,
    pub mobile:Option<String>,
    pub email:String,
    pub tenant_id:Option<i64>,
    pub is_active:bool,
    pub is_staff:bool,
    pub is_admin:bool,
    pub password_change_required:bool,
    pub next_password_change_due:Option<NaiveDate>,
    pub password:String,
    pub last_login:Option<DateTime<Utc>>,
    pub id:i64,
    
}

table! {
    test (id) {
        choice_field ->Nullable<Varchar>,
        bool_field ->Bool,
        #[max_length = 50]
        char_field ->Nullable<Varchar>,
        positive_integer_field ->Integer,
        field3 ->Bool,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable)]

#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = test)]
pub struct Test {
    pub choice_field:Option<String>,
    pub bool_field:bool,
    pub char_field:Option<String>,
    pub positive_integer_field:i32,
    pub field3:bool,
    pub id:i64,
    
}

table! {
    many2ManyTest (id) {
        #[max_length = 200]
        name ->Varchar,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable)]

#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = many2ManyTest)]
pub struct Many2ManyTest {
    pub name:String,
    pub id:i64,
    
}

table! {
    many2ManyTest_testModels (id) {
        id ->Int8,
        test_id ->Int8,
        many2manytest_id ->Int8,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable,Associations)]
#[diesel(belongs_to(Test, foreign_key = test_id))]#[diesel(belongs_to(Many2ManyTest, foreign_key = many2manytest_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = many2ManyTest_testModels)]
pub struct Many2ManyTest_Test {
    pub id:i64,
    pub test_id:i64,
    pub many2manytest_id:i64,
    
}

table! {
    many2ManyTest_crossAppModel (id) {
        id ->Int8,
        testappmodel1_id ->Int8,
        many2manytest_id ->Int8,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable,Associations)]
#[diesel(belongs_to(TestAppModel1, foreign_key = testappmodel1_id))]#[diesel(belongs_to(Many2ManyTest, foreign_key = many2manytest_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = many2ManyTest_crossAppModel)]
pub struct Many2ManyTest_TestAppModel1 {
    pub id:i64,
    pub testappmodel1_id:i64,
    pub many2manytest_id:i64,
    
}

table! {
    dBNamespace (id) {
        tenant_id ->BigInt,
        #[max_length = 200]
        name ->Varchar,
        #[max_length = 200]
        host_env_var_name ->Varchar,
        #[max_length = 200]
        port_env_var_name ->Varchar,
        #[max_length = 200]
        username_env_var_name ->Varchar,
        #[max_length = 200]
        password_env_var_name ->Varchar,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable,Associations)]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = dBNamespace)]
pub struct DBNamespace {
    pub tenant_id:i64,
    pub name:String,
    pub host_env_var_name:String,
    pub port_env_var_name:String,
    pub username_env_var_name:String,
    pub password_env_var_name:String,
    pub id:i64,
    
}

table! {
    languagePack (id) {
        #[max_length = 30]
        lang ->Varchar,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable)]

#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = languagePack)]
pub struct LanguagePack {
    pub lang:String,
    pub id:i64,
    
}

table! {
    template (id) {
        #[max_length = 40]
        kind ->Varchar,
        #[max_length = 500]
        name ->Varchar,
        #[max_length = 100]
        src ->Varchar,
        pack_id ->BigInt,
        #[max_length = 20]
        version ->Varchar,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable,Associations)]
#[diesel(belongs_to(LanguagePack, foreign_key = pack_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = template)]
pub struct Template {
    pub kind:String,
    pub name:String,
    pub src:String,
    pub pack_id:i64,
    pub version:String,
    pub id:i64,
    
}

table! {
    component (id) {
        template_id ->BigInt,
        #[max_length = 100]
        src ->Varchar,
        #[max_length = 100]
        name ->Varchar,
        #[max_length = 500]
        description ->Varchar,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable,Associations)]
#[diesel(belongs_to(Template, foreign_key = template_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = component)]
pub struct Component {
    pub template_id:i64,
    pub src:String,
    pub name:String,
    pub description:String,
    pub id:i64,
    
}

table! {
    serviceMetadata (id) {
        tenant_id ->BigInt,
        #[max_length = 100]
        prod_url ->Varchar,
        #[max_length = 150]
        prod_schema_url ->Varchar,
        #[max_length = 100]
        stage_url ->Varchar,
        #[max_length = 150]
        stage_schema_url ->Varchar,
        #[max_length = 50]
        auth_token_env_key ->Varchar,
        #[max_length = 100]
        name ->Varchar,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable,Associations)]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = serviceMetadata)]
pub struct ServiceMetadata {
    pub tenant_id:i64,
    pub prod_url:String,
    pub prod_schema_url:String,
    pub stage_url:String,
    pub stage_schema_url:String,
    pub auth_token_env_key:String,
    pub name:String,
    pub id:i64,
    
}

table! {
    packageVersion (id) {
        tenant_id ->Nullable<BigInt>,
        #[max_length = 100]
        identifier ->Varchar,
        #[max_length = 20]
        minimum_version ->Varchar,
        #[max_length = 20]
        latest_version ->Varchar,
        allow_older_version ->Bool,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable,Associations)]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = packageVersion)]
pub struct PackageVersion {
    pub tenant_id:Option<i64>,
    pub identifier:String,
    pub minimum_version:String,
    pub latest_version:String,
    pub allow_older_version:bool,
    pub id:i64,
    
}

table! {
    invitation (id) {
        #[max_length = 100]
        code ->Varchar,
        tenant_id ->BigInt,
        invited_by_id ->BigInt,
        user_created_id ->Nullable<BigInt>,
        expiry ->Date,
        is_staff ->Bool,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable,Associations)]
#[diesel(belongs_to(Tenant, foreign_key = tenant_id))]#[diesel(belongs_to(Administrator, foreign_key = invited_by_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = invitation)]
pub struct Invitation {
    pub code:String,
    pub tenant_id:i64,
    pub invited_by_id:i64,
    pub user_created_id:Option<i64>,
    pub expiry:NaiveDate,
    pub is_staff:bool,
    pub id:i64,
    
}

table! {
    testAppModel1 (id) {
        #[max_length = 200]
        char_field ->Varchar,
        id ->BigInt,
        
    }
}

#[derive(Queryable, Debug, Selectable, Serialize, JsonSchema,Identifiable)]

#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = testAppModel1)]
pub struct TestAppModel1 {
    pub char_field:String,
    pub id:i64,
    
}




    diesel::joinable!(django_admin_log -> administrator (user_id));diesel::joinable!(django_admin_log -> django_content_type (content_type_id));

    diesel::joinable!(auth_permission -> django_content_type (content_type_id));

    

    

    

    

    

    diesel::joinable!(administrator -> tenant (tenant_id));

    

    

    

    

    diesel::joinable!(dBNamespace -> tenant (tenant_id));

    

    diesel::joinable!(template -> languagePack (pack_id));

    diesel::joinable!(component -> template (template_id));

    diesel::joinable!(serviceMetadata -> tenant (tenant_id));

    diesel::joinable!(packageVersion -> tenant (tenant_id));

    diesel::joinable!(invitation -> tenant (tenant_id));diesel::joinable!(invitation -> administrator (invited_by_id));

    


diesel::allow_tables_to_appear_in_same_query!(
    django_admin_log,
    auth_permission,
    auth_group,
    auth_group_permissions,
    django_content_type,
    django_session,
    tenant,
    administrator,
    test,
    many2ManyTest,
    many2ManyTest_testModels,
    many2ManyTest_crossAppModel,
    dBNamespace,
    languagePack,
    template,
    component,
    serviceMetadata,
    packageVersion,
    invitation,
    testAppModel1,
    
);
