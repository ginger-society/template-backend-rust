// @generated automatically by Diesel CLI.

diesel::table! {
    administrator (id) {
        id -> Int8,
        #[max_length = 128]
        password -> Varchar,
        last_login -> Nullable<Timestamptz>,
        #[max_length = 200]
        name -> Varchar,
        #[max_length = 20]
        mobile -> Nullable<Varchar>,
        #[max_length = 255]
        email -> Varchar,
        is_active -> Bool,
        is_staff -> Bool,
        is_admin -> Bool,
        password_change_required -> Bool,
        next_password_change_due -> Nullable<Date>,
        tenant_id -> Nullable<Int8>,
    }
}

diesel::table! {
    auth_group (id) {
        id -> Int4,
        #[max_length = 150]
        name -> Varchar,
    }
}

diesel::table! {
    auth_group_permissions (id) {
        id -> Int8,
        group_id -> Int4,
        permission_id -> Int4,
    }
}

diesel::table! {
    auth_permission (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        content_type_id -> Int4,
        #[max_length = 100]
        codename -> Varchar,
    }
}

diesel::table! {
    books (id) {
        id -> Int4,
        title -> Varchar,
    }
}

diesel::table! {
    component (id) {
        id -> Int8,
        #[max_length = 100]
        src -> Varchar,
        #[max_length = 100]
        name -> Varchar,
        description -> Text,
        template_id -> Int8,
    }
}

diesel::table! {
    dBNamespace (id) {
        id -> Int8,
        #[max_length = 200]
        name -> Varchar,
        #[max_length = 200]
        host_env_var_name -> Varchar,
        #[max_length = 200]
        port_env_var_name -> Varchar,
        #[max_length = 200]
        username_env_var_name -> Varchar,
        #[max_length = 200]
        password_env_var_name -> Varchar,
        tenant_id -> Int8,
    }
}

diesel::table! {
    django_admin_log (id) {
        id -> Int4,
        action_time -> Timestamptz,
        object_id -> Nullable<Text>,
        #[max_length = 200]
        object_repr -> Varchar,
        action_flag -> Int2,
        change_message -> Text,
        content_type_id -> Nullable<Int4>,
        user_id -> Int8,
    }
}

diesel::table! {
    django_content_type (id) {
        id -> Int4,
        #[max_length = 100]
        app_label -> Varchar,
        #[max_length = 100]
        model -> Varchar,
    }
}

diesel::table! {
    django_migrations (id) {
        id -> Int8,
        #[max_length = 255]
        app -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        applied -> Timestamptz,
    }
}

diesel::table! {
    django_session (session_key) {
        #[max_length = 40]
        session_key -> Varchar,
        session_data -> Text,
        expire_date -> Timestamptz,
    }
}

diesel::table! {
    invitation (id) {
        id -> Int8,
        #[max_length = 100]
        code -> Varchar,
        expiry -> Date,
        invited_by_id -> Int8,
        tenant_id -> Int8,
        user_created_id -> Nullable<Int8>,
        is_staff -> Bool,
    }
}

diesel::table! {
    languagePack (id) {
        id -> Int8,
        #[max_length = 30]
        lang -> Varchar,
    }
}

diesel::table! {
    many2ManyTest (id) {
        id -> Int8,
        #[max_length = 200]
        name -> Varchar,
    }
}

diesel::table! {
    many2ManyTest_crossAppModel (id) {
        id -> Int8,
        many2manytest_id -> Int8,
        testappmodel1_id -> Int8,
    }
}

diesel::table! {
    many2ManyTest_testModels (id) {
        id -> Int8,
        many2manytest_id -> Int8,
        test_id -> Int8,
    }
}

diesel::table! {
    packageVersion (id) {
        id -> Int8,
        #[max_length = 100]
        identifier -> Varchar,
        #[max_length = 20]
        minimum_version -> Varchar,
        #[max_length = 20]
        latest_version -> Varchar,
        allow_older_version -> Bool,
        tenant_id -> Nullable<Int8>,
    }
}

diesel::table! {
    pages (id) {
        id -> Int4,
        page_number -> Int4,
        content -> Text,
        book_id -> Int4,
        parent_id -> Int4,
    }
}

diesel::table! {
    serviceMetadata (id) {
        id -> Int8,
        #[max_length = 100]
        prod_url -> Varchar,
        #[max_length = 150]
        prod_schema_url -> Varchar,
        #[max_length = 100]
        stage_url -> Varchar,
        #[max_length = 150]
        stage_schema_url -> Varchar,
        #[max_length = 50]
        auth_token_env_key -> Varchar,
        tenant_id -> Int8,
        #[max_length = 100]
        name -> Varchar,
    }
}

diesel::table! {
    template (id) {
        id -> Int8,
        #[max_length = 40]
        kind -> Varchar,
        name -> Text,
        #[max_length = 100]
        src -> Varchar,
        pack_id -> Int8,
        #[max_length = 20]
        version -> Varchar,
    }
}

diesel::table! {
    tenant (id) {
        id -> Int8,
        #[max_length = 200]
        name -> Varchar,
        is_active -> Bool,
        expiry_date -> Nullable<Date>,
    }
}

diesel::table! {
    test (id) {
        id -> Int8,
        #[max_length = 50]
        choice_field -> Nullable<Varchar>,
        bool_field -> Bool,
        #[max_length = 50]
        char_field -> Nullable<Varchar>,
        positive_integer_field -> Int4,
        field3 -> Bool,
    }
}

diesel::table! {
    testAppModel1 (id) {
        id -> Int8,
        #[max_length = 200]
        char_field -> Varchar,
    }
}

diesel::joinable!(administrator -> tenant (tenant_id));
diesel::joinable!(auth_group_permissions -> auth_group (group_id));
diesel::joinable!(auth_group_permissions -> auth_permission (permission_id));
diesel::joinable!(auth_permission -> django_content_type (content_type_id));
diesel::joinable!(component -> template (template_id));
diesel::joinable!(dBNamespace -> tenant (tenant_id));
diesel::joinable!(django_admin_log -> administrator (user_id));
diesel::joinable!(django_admin_log -> django_content_type (content_type_id));
diesel::joinable!(invitation -> tenant (tenant_id));
diesel::joinable!(many2ManyTest_crossAppModel -> many2ManyTest (many2manytest_id));
diesel::joinable!(many2ManyTest_crossAppModel -> testAppModel1 (testappmodel1_id));
diesel::joinable!(many2ManyTest_testModels -> many2ManyTest (many2manytest_id));
diesel::joinable!(many2ManyTest_testModels -> test (test_id));
diesel::joinable!(packageVersion -> tenant (tenant_id));
diesel::joinable!(pages -> books (book_id));
diesel::joinable!(pages -> books (parent_id));
diesel::joinable!(serviceMetadata -> tenant (tenant_id));
diesel::joinable!(template -> languagePack (pack_id));

diesel::allow_tables_to_appear_in_same_query!(
    administrator,
    auth_group,
    auth_group_permissions,
    auth_permission,
    books,
    component,
    dBNamespace,
    django_admin_log,
    django_content_type,
    django_migrations,
    django_session,
    invitation,
    languagePack,
    many2ManyTest,
    many2ManyTest_crossAppModel,
    many2ManyTest_testModels,
    packageVersion,
    pages,
    serviceMetadata,
    template,
    tenant,
    test,
    testAppModel1,
);
