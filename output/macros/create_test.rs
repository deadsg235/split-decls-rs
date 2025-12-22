create_test ! { workspace_deps , r#"
            [dependencies]
            my_crate_cool = { workspace = true }
        "# , r#"
            [workspace.dependencies]
            my_crate_cool = { package = "my_crate" }
        "# , Ok (Some (FoundCrate :: Name (name))) if name == "my_crate_cool" }