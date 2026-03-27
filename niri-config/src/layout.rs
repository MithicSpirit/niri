use std::str::FromStr;

use knuffel::errors::DecodeError;
use niri_ipc::{ColumnDisplay, SizeChange};

use crate::appearance::{
    Border, FocusRing, InsertHint, Shadow, TabIndicator, DEFAULT_BACKGROUND_COLOR,
};
use crate::utils::{expect_only_children, Flag, MergeWith, Percent};
use crate::{BorderRule, Color, FloatOrInt, InsertHintPart, ShadowRule, TabIndicatorPart};

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub focus_ring: FocusRing,
    pub border: Border,
    pub shadow: Shadow,
    pub tab_indicator: TabIndicator,
    pub insert_hint: InsertHint,
    pub preset_column_widths: Vec<PresetSize>,
    pub default_column_width: Option<PresetSize>,
    pub preset_window_heights: Vec<PresetSize>,
    pub center_focused_column: CenterFocusedColumn,
    pub always_center_single_column: bool,
    pub empty_workspace_above_first: bool,
    pub default_column_display: ColumnDisplay,
    pub gaps: f64,
    pub struts: Struts,
    pub background_color: Color,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            focus_ring: FocusRing::default(),
            border: Border::default(),
            shadow: Shadow::default(),
            tab_indicator: TabIndicator::default(),
            insert_hint: InsertHint::default(),
            preset_column_widths: vec![
                PresetSize::Proportion(1. / 3.),
                PresetSize::Proportion(0.5),
                PresetSize::Proportion(2. / 3.),
            ],
            default_column_width: Some(PresetSize::Proportion(0.5)),
            center_focused_column: CenterFocusedColumn::Never,
            always_center_single_column: false,
            empty_workspace_above_first: false,
            default_column_display: ColumnDisplay::Normal,
            gaps: 16.,
            struts: Struts::default(),
            preset_window_heights: vec![
                PresetSize::Proportion(1. / 3.),
                PresetSize::Proportion(0.5),
                PresetSize::Proportion(2. / 3.),
            ],
            background_color: DEFAULT_BACKGROUND_COLOR,
        }
    }
}

impl MergeWith<LayoutPart> for Layout {
    fn merge_with(&mut self, part: &LayoutPart) {
        merge!(
            (self, part),
            focus_ring,
            border,
            shadow,
            tab_indicator,
            insert_hint,
            always_center_single_column,
            empty_workspace_above_first,
            gaps,
        );

        merge_clone!(
            (self, part),
            preset_column_widths,
            preset_window_heights,
            center_focused_column,
            default_column_display,
            struts,
            background_color,
        );

        if let Some(x) = part.default_column_width {
            self.default_column_width = x.0;
        }

        if self.preset_column_widths.is_empty() {
            self.preset_column_widths = Layout::default().preset_column_widths;
        }

        if self.preset_window_heights.is_empty() {
            self.preset_window_heights = Layout::default().preset_window_heights;
        }
    }
}

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct LayoutPart {
    #[knuffel(child)]
    pub focus_ring: Option<BorderRule>,
    #[knuffel(child)]
    pub border: Option<BorderRule>,
    #[knuffel(child)]
    pub shadow: Option<ShadowRule>,
    #[knuffel(child)]
    pub tab_indicator: Option<TabIndicatorPart>,
    #[knuffel(child)]
    pub insert_hint: Option<InsertHintPart>,
    #[knuffel(child, unwrap(children))]
    pub preset_column_widths: Option<Vec<PresetSize>>,
    #[knuffel(child)]
    pub default_column_width: Option<DefaultPresetSize>,
    #[knuffel(child, unwrap(children))]
    pub preset_window_heights: Option<Vec<PresetSize>>,
    #[knuffel(child, unwrap(argument))]
    pub center_focused_column: Option<CenterFocusedColumn>,
    #[knuffel(child)]
    pub always_center_single_column: Option<Flag>,
    #[knuffel(child)]
    pub empty_workspace_above_first: Option<Flag>,
    #[knuffel(child, unwrap(argument, str))]
    pub default_column_display: Option<ColumnDisplay>,
    #[knuffel(child, unwrap(argument))]
    pub gaps: Option<FloatOrInt<0, 65535>>,
    #[knuffel(child)]
    pub struts: Option<Struts>,
    #[knuffel(child)]
    pub background_color: Option<Color>,
}

#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq)]
pub enum PresetSize {
    Proportion(#[knuffel(argument)] f64),
    Fixed(#[knuffel(argument)] i32),
}

impl From<PresetSize> for SizeChange {
    fn from(value: PresetSize) -> Self {
        match value {
            PresetSize::Proportion(prop) => SizeChange::SetProportion(prop * 100.),
            PresetSize::Fixed(fixed) => SizeChange::SetFixed(fixed),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DefaultPresetSize(pub Option<PresetSize>);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrutSize {
    /// Size of strut in logical pixels.
    Pixels(FloatOrInt<-65535, 65535>),
    /// Size of strut as a proportion of the size of the working area.
    Proportion(Percent),
}

#[derive(knuffel::Decode, Debug, Default, Clone, Copy, PartialEq)]
pub struct Struts {
    #[knuffel(child, unwrap(argument), default)]
    pub left: StrutSize,
    #[knuffel(child, unwrap(argument), default)]
    pub right: StrutSize,
    #[knuffel(child, unwrap(argument), default)]
    pub top: StrutSize,
    #[knuffel(child, unwrap(argument), default)]
    pub bottom: StrutSize,
}

#[derive(knuffel::DecodeScalar, Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum CenterFocusedColumn {
    /// Focusing a column will not center the column.
    #[default]
    Never,
    /// The focused column will always be centered.
    Always,
    /// Focusing a column will center it if it doesn't fit on the screen together with the
    /// previously focused column.
    OnOverflow,
}

impl<S> knuffel::Decode<S> for DefaultPresetSize
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        expect_only_children(node, ctx);

        let mut children = node.children();

        if let Some(child) = children.next() {
            if let Some(unwanted_child) = children.next() {
                ctx.emit_error(DecodeError::unexpected(
                    unwanted_child,
                    "node",
                    "expected no more than one child",
                ));
            }
            PresetSize::decode_node(child, ctx).map(Some).map(Self)
        } else {
            Ok(Self(None))
        }
    }
}

impl Default for StrutSize {
    fn default() -> Self {
        Self::Pixels(FloatOrInt(0.))
    }
}

impl From<FloatOrInt<-65535, 65535>> for StrutSize {
    fn from(value: FloatOrInt<-65535, 65535>) -> Self {
        Self::Pixels(value)
    }
}

impl From<Percent> for StrutSize {
    fn from(value: Percent) -> Self {
        Self::Proportion(value)
    }
}

impl MergeWith<FloatOrInt<-65535, 65535>> for StrutSize {
    fn merge_with(&mut self, part: &FloatOrInt<-65535, 65535>) {
        *self = (*part).into();
    }

    fn from_part(part: &FloatOrInt<-65535, 65535>) -> Self
    where
        Self: Default + Sized,
    {
        (*part).into()
    }
}

impl MergeWith<Percent> for StrutSize {
    fn merge_with(&mut self, part: &Percent) {
        *self = (*part).into();
    }

    fn from_part(part: &Percent) -> Self
    where
        Self: Default + Sized,
    {
        (*part).into()
    }
}

impl<S: knuffel::traits::ErrorSpan> knuffel::DecodeScalar<S> for StrutSize {
    fn type_check(
        type_name: &Option<knuffel::span::Spanned<knuffel::ast::TypeName, S>>,
        ctx: &mut knuffel::decode::Context<S>,
    ) {
        if let Some(type_name) = &type_name {
            ctx.emit_error(DecodeError::unexpected(
                type_name,
                "type name",
                "no type name expected for this node",
            ));
        }
    }

    fn raw_decode(
        val: &knuffel::span::Spanned<knuffel::ast::Literal, S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        const MIN: i32 = -65535;
        const MAX: i32 = 65535;
        match &**val {
            knuffel::ast::Literal::Int(ref value) => match value.try_into() {
                Ok(v) => {
                    if (MIN..=MAX).contains(&v) {
                        Ok(Self::Pixels(FloatOrInt(f64::from(v))))
                    } else {
                        ctx.emit_error(DecodeError::conversion(
                            val,
                            format!("value must be between {MIN} and {MAX}"),
                        ));
                        Ok(Self::Pixels(FloatOrInt::default()))
                    }
                }
                Err(e) => {
                    ctx.emit_error(DecodeError::conversion(val, e));
                    Ok(Self::default())
                }
            },
            knuffel::ast::Literal::Decimal(ref value) => match value.try_into() {
                Ok(v) => {
                    if (f64::from(MIN)..=f64::from(MAX)).contains(&v) {
                        Ok(Self::Pixels(FloatOrInt(v)))
                    } else {
                        ctx.emit_error(DecodeError::conversion(
                            val,
                            format!("value must be between {MIN} and {MAX}"),
                        ));
                        Ok(Self::default())
                    }
                }
                Err(e) => {
                    ctx.emit_error(DecodeError::conversion(val, e));
                    Ok(Self::default())
                }
            },
            knuffel::ast::Literal::String(ref value) => match Percent::from_str(value) {
                Ok(v) => Ok(Self::Proportion(v)),
                Err(e) => {
                    ctx.emit_error(DecodeError::conversion(val, e));
                    Ok(Self::default())
                }
            },
            _ => {
                ctx.emit_error(DecodeError::unsupported(
                    val,
                    "Unsupported value, only numbers and strings are recognized",
                ));
                Ok(Self::default())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;

    #[track_caller]
    fn do_parse(text: &str) -> Layout {
        let part = knuffel::parse("test.kdl", text)
            .map_err(miette::Report::new)
            .unwrap();
        Layout::from_part(&part)
    }

    #[test]
    fn strut_pixels() {
        let parsed = do_parse(
            r#"
            struts {
                left 1
                right 1.2
                top 1.23
                bottom 1.234
            }
            "#,
        );
        assert_debug_snapshot!(parsed.struts, @r"
        Struts {
            left: Pixels(
                FloatOrInt(
                    1.0,
                ),
            ),
            right: Pixels(
                FloatOrInt(
                    1.2,
                ),
            ),
            top: Pixels(
                FloatOrInt(
                    1.23,
                ),
            ),
            bottom: Pixels(
                FloatOrInt(
                    1.234,
                ),
            ),
        }
        ")
    }

    #[test]
    fn strut_percent() {
        let parsed = do_parse(
            r#"
            struts {
                left "10%"
                right "12%"
                top "12.3%"
                bottom "12.34%"
            }
            "#,
        );
        assert_debug_snapshot!(parsed.struts, @r"
        Struts {
            left: Proportion(
                Percent(
                    0.1,
                ),
            ),
            right: Proportion(
                Percent(
                    0.12,
                ),
            ),
            top: Proportion(
                Percent(
                    0.12300000000000001,
                ),
            ),
            bottom: Proportion(
                Percent(
                    0.1234,
                ),
            ),
        }
        ")
    }
}
