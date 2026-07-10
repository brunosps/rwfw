use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, FnArg, ImplItem, ImplItemFn, ItemImpl, LitStr, Pat, Type,
    parse_macro_input,
};

pub fn derive_reactive_component(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    match derive_reactive_component_inner(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn derive_reactive_component_inner(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = input.ident;
    if !matches!(input.data, Data::Struct(_)) {
        return Err(syn::Error::new_spanned(
            ident,
            "ReactiveComponent can only be derived for structs",
        ));
    }

    let mut name = None;
    let mut template = None;
    for attr in &input.attrs {
        if attr.path().is_ident("reactive") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    name = Some(meta.value()?.parse::<LitStr>()?);
                    Ok(())
                } else if meta.path.is_ident("template") {
                    template = Some(meta.value()?.parse::<LitStr>()?);
                    Ok(())
                } else {
                    Err(meta.error("unsupported #[reactive(...)] attribute"))
                }
            })?;
        }
    }

    let name =
        name.ok_or_else(|| syn::Error::new_spanned(&ident, "missing #[reactive(name = \"...\")]"))?;
    let template = template.ok_or_else(|| {
        syn::Error::new_spanned(&ident, "missing #[reactive(template = \"...\")]")
    })?;

    Ok(quote! {
        impl ::rwfw_core::reactive::ReactiveComponentMeta for #ident {
            const NAME: &'static str = #name;
            const TEMPLATE: &'static str = #template;
        }
    })
}

pub fn reactive_actions(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    match reactive_actions_inner(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn reactive_actions_inner(mut input: ItemImpl) -> syn::Result<proc_macro2::TokenStream> {
    let self_ty = (*input.self_ty).clone();
    let type_ident = type_ident(&self_ty)?;
    let mut actions = Vec::new();

    for item in &mut input.items {
        if let ImplItem::Fn(method) = item {
            if take_action_attr(&mut method.attrs) {
                actions.push(action_tokens(type_ident, method)?);
            }
        }
    }

    let allowed: Vec<_> = actions.iter().map(|action| &action.name_lit).collect();
    let param_structs: Vec<_> = actions.iter().map(|action| &action.param_struct).collect();
    let match_arms: Vec<_> = actions.iter().map(|action| &action.match_arm).collect();

    Ok(quote! {
        #input

        impl ::rwfw_core::reactive::ReactiveComponent for #self_ty {
            const NAME: &'static str =
                <Self as ::rwfw_core::reactive::ReactiveComponentMeta>::NAME;

            fn template(&self) -> &'static str {
                <Self as ::rwfw_core::reactive::ReactiveComponentMeta>::TEMPLATE
            }

            fn dom_id(&self) -> ::std::string::String {
                <Self as ::rwfw_core::reactive::ReactiveComponentMeta>::dom_id(self)
            }

            fn context(&self) -> ::rwfw_core::reactive::minijinja::Value {
                <Self as ::rwfw_core::reactive::ReactiveComponentMeta>::context(self)
            }

            fn dispatch(
                &mut self,
                action: &str,
                params: ::serde_json::Value,
                ctx: &::rwfw_core::reactive::Ctx<'_>,
            ) -> ::std::result::Result<
                ::rwfw_core::reactive::Reply,
                ::rwfw_core::reactive::ReactiveError,
            > {
                #(#param_structs)*
                let _ = &params;
                match action {
                    #(#match_arms)*
                    _ => Err(::rwfw_core::reactive::ReactiveError::UnknownAction {
                        action: action.to_string(),
                        allowed: &[#(#allowed),*],
                    }),
                }
            }
        }

        ::inventory::submit! {
            ::rwfw_core::reactive::ReactiveRegistration::new::<#self_ty>()
        }
    })
}

struct ActionExpansion {
    name_lit: LitStr,
    param_struct: proc_macro2::TokenStream,
    match_arm: proc_macro2::TokenStream,
}

fn action_tokens(type_ident: &syn::Ident, method: &ImplItemFn) -> syn::Result<ActionExpansion> {
    let method_ident = &method.sig.ident;
    let name = method_ident.to_string();
    let name_lit = LitStr::new(&name, method_ident.span());
    let params = action_params(method)?;
    let struct_ident = format_ident!("__Rwfw{}{}Params", type_ident, to_pascal(&name));

    let (param_struct, parse_params, call_args) = if params.is_empty() {
        (quote! {}, quote! {}, quote! {})
    } else {
        let fields = params.iter().map(|param| {
            let ident = &param.ident;
            let ty = &param.ty;
            quote! { #ident: #ty }
        });
        let call_args = params.iter().map(|param| {
            let ident = &param.ident;
            quote! { __rwfw_params.#ident }
        });
        (
            quote! {
                #[derive(::serde::Deserialize)]
                struct #struct_ident {
                    #(pub #fields,)*
                }
            },
            quote! {
                let __rwfw_params: #struct_ident = ::serde_json::from_value(params)
                    .map_err(|error| ::rwfw_core::reactive::ReactiveError::Invalid(error.to_string()))?;
            },
            quote! { , #(#call_args),* },
        )
    };

    let match_arm = quote! {
        #name_lit => {
            #parse_params
            let __rwfw_action_result = self.#method_ident(ctx #call_args);
            ::rwfw_core::reactive::IntoReactiveReply::into_reactive_reply(__rwfw_action_result)
        }
    };

    Ok(ActionExpansion {
        name_lit,
        param_struct,
        match_arm,
    })
}

struct ActionParam<'a> {
    ident: syn::Ident,
    ty: &'a Type,
}

fn action_params(method: &ImplItemFn) -> syn::Result<Vec<ActionParam<'_>>> {
    let mut typed = method.sig.inputs.iter().filter_map(|arg| match arg {
        FnArg::Receiver(_) => None,
        FnArg::Typed(pat_type) => Some(pat_type),
    });
    let _ctx = typed.next().ok_or_else(|| {
        syn::Error::new_spanned(
            &method.sig.ident,
            "reactive actions must accept a Ctx argument",
        )
    })?;

    typed
        .map(|pat_type| {
            let Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
                return Err(syn::Error::new_spanned(
                    &pat_type.pat,
                    "reactive action params must be simple identifiers",
                ));
            };
            Ok(ActionParam {
                ident: pat_ident.ident.clone(),
                ty: &pat_type.ty,
            })
        })
        .collect()
}

fn take_action_attr(attrs: &mut Vec<Attribute>) -> bool {
    let before = attrs.len();
    attrs.retain(|attr| !attr.path().is_ident("action"));
    attrs.len() != before
}

fn type_ident(ty: &Type) -> syn::Result<&syn::Ident> {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            return Ok(&segment.ident);
        }
    }
    Err(syn::Error::new_spanned(
        ty,
        "#[reactive_actions] requires a concrete self type",
    ))
}

fn to_pascal(input: &str) -> String {
    let mut out = String::new();
    let mut uppercase = true;
    for ch in input.chars() {
        if ch == '_' {
            uppercase = true;
            continue;
        }
        if uppercase {
            out.extend(ch.to_uppercase());
            uppercase = false;
        } else {
            out.push(ch);
        }
    }
    out
}
