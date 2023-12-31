use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{braced, bracketed, token, Attribute, Ident, LitInt, LitStr, Result, Token, Type};

mod kw {
    syn::custom_keyword!(process_name);
    syn::custom_keyword!(snapshot);
    syn::custom_keyword!(access);
    syn::custom_keyword!(game);
}

#[derive(Debug)]
struct MemoryField {
    name: Ident,
    _colon: Token![:],
    elem_type: Type,
    _at: Token![@],
    _bracket: token::Bracket,
    offsets: Punctuated<LitInt, Token![,]>,
}

impl Parse for MemoryField {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        Ok(Self {
            name: input.parse()?,
            _colon: input.parse()?,
            elem_type: input.parse()?,
            _at: input.parse()?,
            _bracket: bracketed!(content in input),
            offsets: content.parse_terminated(LitInt::parse, Token![,])?,
        })
    }
}

impl MemoryField {
    fn format_offset_docs(&self) -> String {
        let offsets = self
            .offsets
            .iter()
            .map(LitInt::base10_parse)
            .collect::<Result<Vec<u32>>>()
            .unwrap();
        match offsets.len() {
            0 => String::new(),
            1 => format!("This value is located at address `{:#010x}`.", offsets[0]),
            2 => format!(
                "This value is located at address `(*{:#010x}) + {:#04x}`.",
                offsets[0], offsets[1]
            ),
            _ => {
                let (first, rest) = offsets.split_first().unwrap();
                let rest = rest
                    .iter()
                    .map(|offset| format!("{:#04x}", offset))
                    .collect::<Vec<_>>()
                    .join(" => ");
                format!(
                    "This value is found via address chain `{:#010x} => {}`",
                    first, rest
                )
            }
        }
    }

    fn snapshot_field_def(&self, attrs: &[Attribute]) -> TokenStream {
        let name = &self.name;
        let elem_type = &self.elem_type;
        quote! {
            #(#attrs)*
            #name: #elem_type
        }
    }

    fn snapshot_access_fn(&self) -> TokenStream {
        let name = &self.name;
        let elem_type = &self.elem_type;
        quote! {
            pub fn #name(&self) -> #elem_type {
                self.#name
            }
        }
    }

    fn snapshot_create_expr(&self) -> TokenStream {
        let name = &self.name;
        quote! { #name: self.#name()? }
    }

    fn access_field_def(&self, attrs: &[Attribute]) -> TokenStream {
        let name = &self.name;
        let elem_type = &self.elem_type;
        let offset_docs = self.format_offset_docs();
        let span = name.span();

        quote_spanned! {span=>
            #(#attrs)*
            ///
            #[doc = #offset_docs]
            #name: touhou_process::FixedData<#elem_type, touhou_process::LittleEndian<4>>
        }
    }

    fn access_create_expr(&self) -> TokenStream {
        let name = &self.name;
        let offsets = self.offsets.iter();
        let span = self.elem_type.span();

        quote_spanned!(span=> #name: handle.new_fixed_item(&[#(#offsets),*]))
    }

    fn access_fn(&self, attrs: &[Attribute], game: &Ident) -> TokenStream {
        let name = &self.name;
        let elem_type = &self.elem_type;
        let offset_docs = self.format_offset_docs();
        let span = elem_type.span();

        quote_spanned! {span=>
            #(#attrs)*
            ///
            #[doc = #offset_docs]
            pub fn #name(&self) -> Result<#elem_type, crate::memory::MemoryReadError<#game>> {
                use crate::memory::MemoryReadError;
                self.#name.read().map_err(MemoryReadError::from)
            }
        }
    }

    fn wrapper_access_fn(&self, attrs: &[Attribute], game: &Ident) -> TokenStream {
        let name = &self.name;
        let elem_type = &self.elem_type;
        let offset_docs = self.format_offset_docs();

        quote! {
            #(#attrs)*
            ///
            #[doc = #offset_docs]
            pub fn #name(&mut self) -> Result<Option<#elem_type>, crate::memory::MemoryReadError<#game>> {
                self.0.access().map(|inner| inner.#name()).transpose()
            }
        }
    }
}

#[derive(Debug)]
enum MemoryDefElement {
    ProcessName {
        _kw: kw::process_name,
        _eq: Token![=],
        name: LitStr,
    },
    SnapshotType {
        attrs: Vec<Attribute>,
        _kw: kw::snapshot,
        _eq: Token![=],
        name: Ident,
    },
    AccessType {
        attrs: Vec<Attribute>,
        _kw: kw::access,
        _eq: Token![=],
        name: Ident,
    },
    GameType {
        _attrs: Vec<Attribute>,
        _kw: kw::game,
        _eq: Token![=],
        name: Ident,
    },
    Field {
        attrs: Vec<Attribute>,
        field: MemoryField,
    },
}

impl Parse for MemoryDefElement {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let lookahead = input.lookahead1();
        if lookahead.peek(kw::process_name) {
            Ok(Self::ProcessName {
                _kw: input.parse()?,
                _eq: input.parse()?,
                name: input.parse()?,
            })
        } else if lookahead.peek(kw::snapshot) {
            Ok(Self::SnapshotType {
                attrs,
                _kw: input.parse()?,
                _eq: input.parse()?,
                name: input.parse()?,
            })
        } else if lookahead.peek(kw::access) {
            Ok(Self::AccessType {
                attrs,
                _kw: input.parse()?,
                _eq: input.parse()?,
                name: input.parse()?,
            })
        } else if lookahead.peek(kw::game) {
            Ok(Self::GameType {
                _attrs: attrs,
                _kw: input.parse()?,
                _eq: input.parse()?,
                name: input.parse()?,
            })
        } else {
            Ok(Self::Field {
                attrs,
                field: input.parse()?,
            })
        }
    }
}

#[derive(Debug)]
struct MemoryDefAST {
    attrs: Vec<Attribute>,
    name: Ident,
    _brace: token::Brace,
    elems: Punctuated<MemoryDefElement, Token![,]>,
}

impl Parse for MemoryDefAST {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            name: input.parse()?,
            _brace: braced!(content in input),
            elems: content.parse_terminated(MemoryDefElement::parse, Token![,])?,
        })
    }
}

#[derive(Debug)]
pub struct MemoryDef {
    attrs: Vec<Attribute>,
    name: Ident,
    snapshot_name: Option<(Vec<Attribute>, Ident)>,
    access_name: (Vec<Attribute>, Ident),
    process_names: Vec<LitStr>,
    game_type: Ident,
    fields: Vec<(Vec<Attribute>, MemoryField)>,
}

impl Parse for MemoryDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let MemoryDefAST {
            attrs: main_attrs,
            name,
            elems,
            ..
        } = MemoryDefAST::parse(input)?;
        let mut snapshot_name = None;
        let mut access_name = None;
        let mut game_type = None;
        let mut process_names = Vec::new();
        let mut fields = Vec::new();

        for elem in elems {
            match elem {
                MemoryDefElement::Field { attrs, field } => fields.push((attrs, field)),
                MemoryDefElement::ProcessName { name, .. } => process_names.push(name),
                MemoryDefElement::SnapshotType { attrs, name, .. } => {
                    if snapshot_name.is_none() {
                        snapshot_name = Some((attrs, name));
                    } else {
                        return Err(syn::Error::new(
                            name.span(),
                            "multiple snapshot type names given",
                        ));
                    }
                }
                MemoryDefElement::AccessType { attrs, name, .. } => {
                    if access_name.is_none() {
                        access_name = Some((attrs, name));
                    } else {
                        return Err(syn::Error::new(
                            name.span(),
                            "multiple access type names given",
                        ));
                    }
                }
                MemoryDefElement::GameType { name, .. } => {
                    if game_type.is_none() {
                        game_type = Some(name);
                    } else {
                        return Err(syn::Error::new(name.span(), "multiple game types given"));
                    }
                }
            }
        }

        if process_names.is_empty() {
            return Err(input.error("no process names given"));
        }

        Ok(Self {
            attrs: main_attrs,
            name,
            snapshot_name,
            access_name: access_name.ok_or_else(|| input.error("no access type name given"))?,
            game_type: game_type.ok_or_else(|| input.error("no game type given"))?,
            process_names,
            fields,
        })
    }
}

impl MemoryDef {
    fn define_snapshot_struct(&self) -> Option<TokenStream> {
        let field_defs = self
            .fields
            .iter()
            .map(|pair| pair.1.snapshot_field_def(&pair.0[..]));
        let field_access = self.fields.iter().map(|pair| pair.1.snapshot_access_fn());

        self.snapshot_name.as_ref().map(|(attrs, snapshot_name)| {
            quote! {
                #(#attrs)*
                pub struct #snapshot_name {
                    #(#field_defs),*
                }

                #[automatically_derived]
                impl #snapshot_name {
                    #(#field_access)*
                }
            }
        })
    }

    fn define_access_struct(&self) -> TokenStream {
        let (access_attrs, access_name) = &self.access_name;
        let game = &self.game_type;
        let field_defs = self
            .fields
            .iter()
            .map(|(attrs, field)| field.access_field_def(attrs));
        let field_create = self
            .fields
            .iter()
            .map(|(_, field)| field.access_create_expr());
        let field_access = self
            .fields
            .iter()
            .map(|(attrs, field)| field.access_fn(attrs, game));
        let (first_name, other_names) = self.process_names.split_first().unwrap();

        let snapshot_create = self.snapshot_name.as_ref().map(|(_, snapshot_name)| {
            let snapshot_fields = self
                .fields
                .iter()
                .map(|(_, field)| field.snapshot_create_expr());

            quote! {
                pub fn read_snapshot(&self) -> Result<#snapshot_name, crate::memory::MemoryReadError<#game>> {
                    Ok(#snapshot_name {
                        #(#snapshot_fields),*
                    })
                }
            }
        });

        quote! {
            #(#access_attrs)*
            pub struct #access_name {
                #(#field_defs),*
            }

            #[automatically_derived]
            impl ProcessAttached for #access_name {
                fn from_pid(pid: u32) -> std::io::Result<Self> {
                    touhou_process::Pid::from(pid).try_into_process_handle().map(|handle| Self {
                        #(#field_create),*
                    })
                }

                fn is_attachable_process(proc: &sysinfo::Process) -> bool {
                    let exe = <sysinfo::Process as sysinfo::ProcessExt>::exe(proc);
                    exe.file_stem().and_then(|s| s.to_str()).is_some_and(|name| name.starts_with(#first_name) #(|| name.starts_with(#other_names))*)
                }
            }

            #[automatically_derived]
            impl #access_name {
                #(#field_access)*

                #snapshot_create
            }
        }
    }

    fn define_wrapper_struct(&self) -> TokenStream {
        let main_attrs = &self.attrs;
        let game = &self.game_type;
        let name = &self.name;
        let name_str = self.name.to_string();
        let (_, access_name) = &self.access_name;
        let field_access = self
            .fields
            .iter()
            .map(|(attrs, field)| field.wrapper_access_fn(attrs, game));

        let snapshot_access = self.snapshot_name.as_ref().map(|(_, snapshot_name)| {
            quote! {
                pub fn read_snapshot(&mut self) -> Result<#snapshot_name, crate::memory::MemoryReadError<#game>> {
                    self.0.access().and_then(|inner| inner.read_snapshot())
                }
            }
        });

        quote! {
            #(#main_attrs)*
            #[repr(transparent)]
            pub struct #name(Attached<#access_name>);

            #[automatically_derived]
            impl std::fmt::Debug for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}(<attached to PID {}>)", #name_str, self.pid())
                }
            }

            #[automatically_derived]
            impl #name {
                pub fn new() -> Result<Option<Self>, crate::memory::MemoryReadError<#game>> {
                    use crate::memory::MemoryReadError;
                    Attached::new().map(|inner| inner.map(Self)).map_err(MemoryReadError::from)
                }

                pub fn from_pid(pid: u32) -> Result<Self, crate::memory::MemoryReadError<#game>> {
                    use crate::memory::MemoryReadError;
                    Attached::from_pid(pid).map(Self).map_err(MemoryReadError::from)
                }

                pub fn is_running(&mut self) -> bool {
                    self.0.is_running()
                }

                pub fn pid(&self) -> u32 {
                    self.0.pid()
                }

                pub fn access(&mut self) -> Option<&#access_name> {
                    self.0.access()
                }

                #(#field_access)*

                #snapshot_access
            }

            impl Clone for #name {
                fn clone(&self) -> Self {
                    Self::from_pid(self.pid()).unwrap()
                }
            }

            impl crate::memory::GameMemory<#game> for #name {
                type MemoryAccess = #access_name;

                fn is_running(&mut self) -> bool {
                    self.0.is_running()
                }

                fn pid(&self) -> u32 {
                    self.0.pid()
                }

                fn access(&mut self) -> Option<&#access_name> {
                    self.0.access()
                }
            }
        }
    }

    pub fn into_defines(self) -> TokenStream {
        let mut ret = self.define_access_struct();
        if let Some(tokens) = self.define_snapshot_struct() {
            ret.extend(tokens)
        }
        ret.extend(self.define_wrapper_struct());
        ret
    }
}
