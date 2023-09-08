use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, bracketed, token, Ident, LitInt, LitStr, Result, Token, Type};

mod kw {
    syn::custom_keyword!(process_name);
    syn::custom_keyword!(snapshot);
    syn::custom_keyword!(access);
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
    fn snapshot_field_def(&self) -> TokenStream {
        let name = &self.name;
        let elem_type = &self.elem_type;
        quote! { #name: #elem_type }
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

    fn access_field_def(&self) -> TokenStream {
        let name = &self.name;
        let elem_type = &self.elem_type;

        quote! { #name: process_memory::DataMember<#elem_type> }
    }

    fn access_create_expr(&self) -> TokenStream {
        let name = &self.name;
        let offsets = self.offsets.iter();

        quote! { #name: process_memory::DataMember::new_offset(handle, vec![#(#offsets),*])}
    }

    fn access_fn(&self) -> TokenStream {
        let name = &self.name;
        let elem_type = &self.elem_type;

        quote! {
            pub fn #name(&self) -> std::io::Result<#elem_type> {
                unsafe { self.#name.read() }
            }
        }
    }

    fn wrapper_access_fn(&self) -> TokenStream {
        let name = &self.name;
        let elem_type = &self.elem_type;

        quote! {
            pub fn #name(&mut self) -> std::io::Result<#elem_type> {
                self.0.access().and_then(|inner| inner.#name())
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
        _kw: kw::snapshot,
        _eq: Token![=],
        name: Ident,
    },
    AccessType {
        _kw: kw::access,
        _eq: Token![=],
        name: Ident,
    },
    Field(MemoryField),
}

impl Parse for MemoryDefElement {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::process_name) {
            Ok(Self::ProcessName {
                _kw: input.parse()?,
                _eq: input.parse()?,
                name: input.parse()?,
            })
        } else if lookahead.peek(kw::snapshot) {
            Ok(Self::SnapshotType {
                _kw: input.parse()?,
                _eq: input.parse()?,
                name: input.parse()?,
            })
        } else if lookahead.peek(kw::access) {
            Ok(Self::AccessType {
                _kw: input.parse()?,
                _eq: input.parse()?,
                name: input.parse()?,
            })
        } else {
            input.parse().map(Self::Field)
        }
    }
}

#[derive(Debug)]
struct MemoryDefAST {
    name: Ident,
    _brace: token::Brace,
    elems: Punctuated<MemoryDefElement, Token![,]>,
}

impl Parse for MemoryDefAST {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        Ok(Self {
            name: input.parse()?,
            _brace: braced!(content in input),
            elems: content.parse_terminated(MemoryDefElement::parse, Token![,])?,
        })
    }
}

#[derive(Debug)]
pub struct MemoryDef {
    name: Ident,
    snapshot_name: Option<Ident>,
    access_name: Ident,
    process_names: Vec<LitStr>,
    fields: Vec<MemoryField>,
}

impl Parse for MemoryDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let MemoryDefAST { name, elems, .. } = MemoryDefAST::parse(input)?;
        let mut snapshot_name = None;
        let mut access_name = None;
        let mut process_names = Vec::new();
        let mut fields = Vec::new();

        for elem in elems {
            match elem {
                MemoryDefElement::Field(field) => fields.push(field),
                MemoryDefElement::ProcessName { name, .. } => process_names.push(name),
                MemoryDefElement::SnapshotType { name, .. } => {
                    if snapshot_name.is_none() {
                        snapshot_name = Some(name);
                    } else {
                        return Err(syn::Error::new(
                            name.span(),
                            "multiple snapshot type names given",
                        ));
                    }
                }
                MemoryDefElement::AccessType { name, .. } => {
                    if access_name.is_none() {
                        access_name = Some(name);
                    } else {
                        return Err(syn::Error::new(
                            name.span(),
                            "multiple access type names given",
                        ));
                    }
                }
            }
        }

        if process_names.is_empty() {
            return Err(input.error("no process names given"));
        }

        if let Some(access_name) = access_name {
            Ok(Self {
                name,
                snapshot_name,
                access_name,
                process_names,
                fields,
            })
        } else {
            Err(input.error("no access type name given"))
        }
    }
}

impl MemoryDef {
    fn define_snapshot_struct(&self) -> Option<TokenStream> {
        let field_defs = self.fields.iter().map(MemoryField::snapshot_field_def);
        let field_access = self.fields.iter().map(MemoryField::snapshot_access_fn);

        self.snapshot_name.as_ref().map(|snapshot_name| {
            quote! {
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
        let access_name = &self.access_name;
        let field_defs = self.fields.iter().map(MemoryField::access_field_def);
        let field_create = self.fields.iter().map(MemoryField::access_create_expr);
        let field_access = self.fields.iter().map(MemoryField::access_fn);
        let (first_name, other_names) = self.process_names.split_first().unwrap();

        let snapshot_create = self.snapshot_name.as_ref().map(|snapshot_name| {
            let snapshot_fields = self.fields.iter().map(MemoryField::snapshot_create_expr);
            quote! {
                pub fn read_snapshot(&self) -> std::io::Result<#snapshot_name> {
                    Ok(#snapshot_name {
                        #(#snapshot_fields),*
                    })
                }
            }
        });

        quote! {
            pub struct #access_name {
                #(#field_defs),*
            }

            #[automatically_derived]
            impl ProcessAttached for #access_name {
                fn from_pid(pid: u32) -> std::io::Result<Self> {
                    let handle = pid
                        .try_into_process_handle()?
                        .set_arch(process_memory::Architecture::Arch32Bit);

                    Ok(Self {
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
        let name = &self.name;
        let name_str = self.name.to_string();
        let access_name = &self.access_name;
        let field_access = self.fields.iter().map(MemoryField::wrapper_access_fn);

        let snapshot_access = self.snapshot_name.as_ref().map(|snapshot_name| {
            quote! {
                pub fn read_snapshot(&mut self) -> std::io::Result<#snapshot_name> {
                    self.0.access().and_then(|inner| inner.read_snapshot())
                }
            }
        });

        quote! {
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
                pub fn new() -> std::io::Result<Option<Self>> {
                    Attached::new().map(|inner| inner.map(Self))
                }

                pub fn is_running(&mut self) -> bool {
                    self.0.is_running()
                }

                pub fn pid(&self) -> u32 {
                    self.0.pid()
                }

                pub fn access(&mut self) -> std::io::Result<&#access_name> {
                    self.0.access()
                }

                #(#field_access)*

                #snapshot_access
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
