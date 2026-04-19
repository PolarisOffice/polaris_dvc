/*
 * Copyright 2022 Hancom Inc. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#pragma once

#include "json.h"
//////////////////////////////////////////////////////////
// INT MIN/MAX
#define JNI_INT_MIN                                            "minimum"
#define JNI_INT_MAX                                            "maximum"


//////////////////////////////////////////////////////////
// 글자모양 : JID_CHARSHAPE
#define JIN_CHAR_SHAPE                                        "charshape"
#define JIN_CHAR_SHAPE_FONTSIZE                                "fontsize"                // int
//#define JIN_CHAR_SHAPE_FONTSIZE_MIN                            "fontsize-min"            // int
//#define JIN_CHAR_SHAPE_FONTSIZE_MAX                            "fontsize-max"            // int
#define JIN_CHAR_SHAPE_LANGSET                                "langset"                // object
#define JIN_CHAR_SHAPE_LANG                                    "langtype"                // int
#define JIN_CHAR_SHAPE_FONT                                    "font"                    // string
#define JIN_CHAR_SHAPE_RSIZE                                "r-size"                // int
#define JIN_CHAR_SHAPE_POSITION                                "position"                // int
#define JIN_CHAR_SHAPE_RATIO                                "ratio"                    // int
#define JIN_CHAR_SHAPE_SPACING                                "spacing"                // int
#define JIN_CHAR_SHAPE_BOLD                                    "bold"                    // bool
#define JIN_CHAR_SHAPE_ITALIC                                "italic"                // bool
#define JIN_CHAR_SHAPE_UNDERLINE                            "underline"                // bool
#define JIN_CHAR_SHAPE_STRIKEOUT                            "strikeout"                // bool
#define JIN_CHAR_SHAPE_OUTLINE                                "outline"                // bool
#define JIN_CHAR_SHAPE_EMBOSS                                "emboss"                // bool
#define JIN_CHAR_SHAPE_ENGRAVE                                "engrave"                // bool
#define JIN_CHAR_SHAPE_SHADOW                                "shadow"                // bool
#define JIN_CHAR_SHAPE_SUPSCRIPT                            "supscript"                // bool
#define JIN_CHAR_SHAPE_SUBSCRIPT                            "subscript"                // bool
#define JIN_CHAR_SHAPE_SHADOWTYPE                            "shadowtype"            // array, string
#define JIN_CHAR_SHAPE_SHADOW_X                                "shadow-x"                // int
#define JIN_CHAR_SHAPE_SHADOW_Y                                "shadow-Y"                // int
#define JIN_CHAR_SHAPE_SHADOW_COLOR                            "shadow-color"            // int
#define JIN_CHAR_SHAPE_UNDERLINE_POSITOION                    "underline-position"    // array, string
#define JIN_CHAR_SHAPE_UNDERLINE_SHAPE                        "underline-shape"        // array, string
#define JIN_CHAR_SHAPE_UNDERLINE_COLOR                        "underline-color"        // int
#define JIN_CHAR_SHAPE_STRIKEOUT_SHAPE                        "strikeout-shape"        // array, string
#define JIN_CHAR_SHAPE_STRIKEOUT_COLOR                        "strikeout-color"        // int
#define JIN_CHAR_SHAPE_OUTLINETYPE                            "outlinetype"            // array, string
#define JIN_CHAR_SHAPE_EMPTYSPACE                            "emptyspace"            // bool
#define JIN_CHAR_SHAPE_POINT                                "point"                    // object
#define JIN_CHAR_SHAPE_KERNING                                "kerning"                // bool
#define JIN_CHAR_SHAPE_BG_BORDER                            "border"                // object
#define JIN_CHAR_SHAPE_BG_BORDER_POSITION                    "position"                // array, string
#define JIN_CHAR_SHAPE_BG_BORDER_BORDERTYPE                    "bordertype"            // array, string
#define JIN_CHAR_SHAPE_BG_BORDER_SIZE                        "size"                    // int
#define JIN_CHAR_SHAPE_BG_BORDER_COLOR                        "color"                    // int
#define JIN_CHAR_SHAPE_BG_COLOR                                "bg-color"                // int    
#define JIN_CHAR_SHAPE_BG_PATTONCOLOR                        "bg-pattoncolor"        // int    
#define JIN_CHAR_SHAPE_BG_PATTONTYPE                        "bg-pattontype"            // array, string



#define    JID_CHAR_SHAPE                                        1000    
#define JID_CHAR_SHAPE_FONTSIZE                                JID_CHAR_SHAPE+1        
#define JID_CHAR_SHAPE_LANGSET                                JID_CHAR_SHAPE+2        
#define JID_CHAR_SHAPE_LANG                                    JID_CHAR_SHAPE+3        
#define JID_CHAR_SHAPE_FONT                                    JID_CHAR_SHAPE+4        
#define JID_CHAR_SHAPE_RSIZE                                JID_CHAR_SHAPE+5        
#define JID_CHAR_SHAPE_POSITION                                JID_CHAR_SHAPE+6        
#define JID_CHAR_SHAPE_RATIO                                JID_CHAR_SHAPE+7        
#define JID_CHAR_SHAPE_SPACING                                JID_CHAR_SHAPE+8        
#define JID_CHAR_SHAPE_BOLD                                    JID_CHAR_SHAPE+9            
#define JID_CHAR_SHAPE_ITALIC                                JID_CHAR_SHAPE+10        
#define JID_CHAR_SHAPE_UNDERLINE                            JID_CHAR_SHAPE+11        
#define JID_CHAR_SHAPE_STRIKEOUT                            JID_CHAR_SHAPE+12        
#define JID_CHAR_SHAPE_OUTLINE                                JID_CHAR_SHAPE+13        
#define JID_CHAR_SHAPE_EMBOSS                                JID_CHAR_SHAPE+14        
#define JID_CHAR_SHAPE_ENGRAVE                                JID_CHAR_SHAPE+15        
#define JID_CHAR_SHAPE_SHADOW                                JID_CHAR_SHAPE+16        
#define JID_CHAR_SHAPE_SUPSCRIPT                            JID_CHAR_SHAPE+17        
#define JID_CHAR_SHAPE_SUBSCRIPT                            JID_CHAR_SHAPE+18        
#define JID_CHAR_SHAPE_SHADOWTYPE                            JID_CHAR_SHAPE+19        
#define JID_CHAR_SHAPE_SHADOW_X                                JID_CHAR_SHAPE+20        
#define JID_CHAR_SHAPE_SHADOW_Y                                JID_CHAR_SHAPE+21        
#define JID_CHAR_SHAPE_SHADOW_COLOR                            JID_CHAR_SHAPE+22        
#define JID_CHAR_SHAPE_UNDERLINE_POSITOION                    JID_CHAR_SHAPE+23        
#define JID_CHAR_SHAPE_UNDERLINE_SHAPE                        JID_CHAR_SHAPE+24        
#define JID_CHAR_SHAPE_UNDERLINE_COLOR                        JID_CHAR_SHAPE+25        
#define JID_CHAR_SHAPE_STRIKEOUT_SHAPE                        JID_CHAR_SHAPE+26        
#define JID_CHAR_SHAPE_STRIKEOUT_COLOR                        JID_CHAR_SHAPE+27        
#define JID_CHAR_SHAPE_OUTLINETYPE                            JID_CHAR_SHAPE+28        
#define JID_CHAR_SHAPE_EMPTYSPACE                            JID_CHAR_SHAPE+29        
#define JID_CHAR_SHAPE_POINT                                JID_CHAR_SHAPE+30        
#define JID_CHAR_SHAPE_KERNING                                JID_CHAR_SHAPE+31        
#define JID_CHAR_SHAPE_BG_BORDER                            JID_CHAR_SHAPE+32        
#define JID_CHAR_SHAPE_BG_BORDER_POSITION                    JID_CHAR_SHAPE+33        
#define JID_CHAR_SHAPE_BG_BORDER_BORDERTYPE                    JID_CHAR_SHAPE+34        
#define JID_CHAR_SHAPE_BG_BORDER_SIZE                        JID_CHAR_SHAPE+35        
#define JID_CHAR_SHAPE_BG_BORDER_COLOR                        JID_CHAR_SHAPE+36        
#define JID_CHAR_SHAPE_BG_COLOR                                JID_CHAR_SHAPE+37        
#define JID_CHAR_SHAPE_BG_PATTONCOLOR                        JID_CHAR_SHAPE+38        
#define JID_CHAR_SHAPE_BG_PATTONTYPE                        JID_CHAR_SHAPE+39        
#define JID_CHAR_SHAPE_END                                    JID_CHAR_SHAPE+40        


//////////////////////////////////////////////////////////
// 문단모양 : JID_PARA_SHAPE
#define JIN_PARA_SHAPE                                        "parashape"
#define JIN_PARA_SHAPE_HORIZONTAL                            "horizontal"
#define JIN_PARA_SHAPE_MARGINLEFT                            "margin-left"
#define JIN_PARA_SHAPE_MARGINRIGHT                            "margin-right"
#define JIN_PARA_SHAPE_FIRSTLINE                            "firstline"
#define JIN_PARA_SHAPE_INDENT                                "indent"
#define JIN_PARA_SHAPE_OUTDENT                                "outdent"
#define JIN_PARA_SHAPE_LINESPACING                            "linespacing"
#define JIN_PARA_SHAPE_LINESPACINGVALUE                        "linespacingvalue"
#define JIN_PARA_SHAPE_SPACINGPARAUP                        "spacing-paraup"
#define JIN_PARA_SHAPE_SPACINGPARABOTTOM                    "spacing-parabottom"
#define JIN_PARA_SHAPE_SPACINGGRIDPAPER                        "spacing-gridpaper"
#define JIN_PARA_SHAPE_LINEBREAKKOREAN                        "linebreak-korean"
#define JIN_PARA_SHAPE_LINEBREAKENGLISH                        "linebreak-english"
#define JIN_PARA_SHAPE_LINEBREAKCONDENSE                    "linebreak-condense"
#define JIN_PARA_SHAPE_PARATYPE                                "paratype"
#define JIN_PARA_SHAPE_PARATYPEVALUE                        "paratype-value"
#define JIN_PARA_SHAPE_WIDOWORPHAN                            "widow-orphan"
#define JIN_PARA_SHAPE_KEEPWITHNEXT                            "keep-with-next"
#define JIN_PARA_SHAPE_KEEPLINESTOGETHER                    "keep-lines-together"
#define JIN_PARA_SHAPE_PAGEBREAKBEFORE                        "pagebreak-before"
#define JIN_PARA_SHAPE_FONTLINEHEIGHT                        "fontlineheight"
#define JIN_PARA_SHAPE_LINEWRAP                                "linewrap"
#define JIN_PARA_SHAPE_AUTOSPACEEASIANENG                    "autospace-easian-eng"
#define JIN_PARA_SHAPE_AUTOSPACEEASIANNUM                    "autospace-easian-num"
#define JIN_PARA_SHAPE_VERTICALALIGN                        "verticalalign"
#define JIN_PARA_SHAPE_TABTYPES                                "tabtypes"
#define JIN_PARA_SHAPE_TABTYPE                                "type"
#define JIN_PARA_SHAPE_TABSHAPE                                "shape"
#define JIN_PARA_SHAPE_TABPOSITION                            "position"
#define JIN_PARA_SHAPE_AUTOTABINDENT                        "autotab-intent"
#define JIN_PARA_SHAPE_AUTOTABPARARIGHTEND                    "autotab-pararightend"
#define JIN_PARA_SHAPE_BASETABSPACE                            "basetabspace"
#define JIN_PARA_SHAPE_BORDER                                "border"
#define JIN_PARA_SHAPE_BORDERPOSITION                        "position"
#define JIN_PARA_SHAPE_BORDERTYPE                            "bordertype"
#define JIN_PARA_SHAPE_BORDERSIZE                            "size"
#define JIN_PARA_SHAPE_BORDERCOLOR                            "color"
#define JIN_PARA_SHAPE_BGCOLOR                                "bg-color"
#define JIN_PARA_SHAPE_BGPATTONCOLOR                        "bg-pattoncolor"
#define JIN_PARA_SHAPE_BGPATTONTYPE                            "bg-pattontype"
#define JIN_PARA_SHAPE_SPACINGLEFT                            "spacing-left"
#define JIN_PARA_SHAPE_SPACINGRIGHT                            "spacing-right"
#define JIN_PARA_SHAPE_SPACINGTOP                            "spacing-top"
#define JIN_PARA_SHAPE_SPACINGBOTTOM                        "spacing-bottom"
#define JIN_PARA_SHAPE_SPACINGIGNORE                        "spacing-ignore"


#define JID_PARA_SHAPE                                        2000
#define JID_PARA_SHAPE_HORIZONTAL                            JID_PARA_SHAPE+1
#define JID_PARA_SHAPE_MARGINLEFT                            JID_PARA_SHAPE+2
#define JID_PARA_SHAPE_MARGINRIGHT                            JID_PARA_SHAPE+3
#define JID_PARA_SHAPE_FIRSTLINE                            JID_PARA_SHAPE+4
#define JID_PARA_SHAPE_INDENT                                JID_PARA_SHAPE+5
#define JID_PARA_SHAPE_OUTDENT                                JID_PARA_SHAPE+6
#define JID_PARA_SHAPE_LINESPACING                            JID_PARA_SHAPE+7
#define JID_PARA_SHAPE_LINESPACINGVALUE                        JID_PARA_SHAPE+8
#define JID_PARA_SHAPE_SPACINGPARAUP                        JID_PARA_SHAPE+9
#define JID_PARA_SHAPE_SPACINGPARABOTTOM                    JID_PARA_SHAPE+10
#define JID_PARA_SHAPE_SPACINGGRIDPAPER                        JID_PARA_SHAPE+11
#define JID_PARA_SHAPE_LINEBREAKKOREAN                        JID_PARA_SHAPE+12
#define JID_PARA_SHAPE_LINEBREAKENGLISH                        JID_PARA_SHAPE+13
#define JID_PARA_SHAPE_LINEBREAKCONDENSE                    JID_PARA_SHAPE+14
#define JID_PARA_SHAPE_PARATYPE                                JID_PARA_SHAPE+15
#define JID_PARA_SHAPE_PARATYPEVALUE                        JID_PARA_SHAPE+16
#define JID_PARA_SHAPE_WIDOWORPHAN                            JID_PARA_SHAPE+17
#define JID_PARA_SHAPE_KEEPWITHNEXT                            JID_PARA_SHAPE+18
#define JID_PARA_SHAPE_KEEPLINESTOGETHER                    JID_PARA_SHAPE+19
#define JID_PARA_SHAPE_PAGEBREAKBEFORE                        JID_PARA_SHAPE+20
#define JID_PARA_SHAPE_FONTLINEHEIGHT                        JID_PARA_SHAPE+21
#define JID_PARA_SHAPE_LINEWRAP                                JID_PARA_SHAPE+22
#define JID_PARA_SHAPE_AUTOSPACEEASIANENG                    JID_PARA_SHAPE+23
#define JID_PARA_SHAPE_AUTOSPACEEASIANNUM                    JID_PARA_SHAPE+24
#define JID_PARA_SHAPE_VERTICALALIGN                        JID_PARA_SHAPE+25
#define JID_PARA_SHAPE_TABTYPES                                JID_PARA_SHAPE+26
#define JID_PARA_SHAPE_TABTYPE                                JID_PARA_SHAPE+27
#define JID_PARA_SHAPE_TABSHAPE                                JID_PARA_SHAPE+28
#define JID_PARA_SHAPE_TABPOSITION                            JID_PARA_SHAPE+29
#define JID_PARA_SHAPE_AUTOTABINDENT                        JID_PARA_SHAPE+30
#define JID_PARA_SHAPE_AUTOTABPARARIGHTEND                    JID_PARA_SHAPE+31
#define JID_PARA_SHAPE_BASETABSPACE                            JID_PARA_SHAPE+32
#define JID_PARA_SHAPE_BORDER                                JID_PARA_SHAPE+33
#define JID_PARA_SHAPE_BORDERPOSITION                        JID_PARA_SHAPE+34
#define JID_PARA_SHAPE_BORDERTYPE                            JID_PARA_SHAPE+35
#define JID_PARA_SHAPE_BORDERSIZE                            JID_PARA_SHAPE+36
#define JID_PARA_SHAPE_BORDERCOLOR                            JID_PARA_SHAPE+37
#define JID_PARA_SHAPE_BGCOLOR                                JID_PARA_SHAPE+38
#define JID_PARA_SHAPE_BGPATTONCOLOR                        JID_PARA_SHAPE+39
#define JID_PARA_SHAPE_BGPATTONTYPE                            JID_PARA_SHAPE+40
#define JID_PARA_SHAPE_SPACINGLEFT                            JID_PARA_SHAPE+41
#define JID_PARA_SHAPE_SPACINGRIGHT                            JID_PARA_SHAPE+42
#define JID_PARA_SHAPE_SPACINGTOP                            JID_PARA_SHAPE+43
#define JID_PARA_SHAPE_SPACINGBOTTOM                        JID_PARA_SHAPE+44
#define JID_PARA_SHAPE_SPACINGIGNORE                        JID_PARA_SHAPE+45
#define JID_PARA_SHAPE_END                                    JID_PARA_SHAPE+999



//////////////////////////////////////////////////////////
// 표 : JID_TABLE
#define    JIN_TABLE                                            "table"
#define    JIN_TABLE_SIZEWIDTH                                    "size-width"
#define    JIN_TABLE_SIZEHEIGHT                                "size-height"
#define    JIN_TABLE_SIZEFIXED                                    "fixed"
#define    JIN_TABLE_TREATASCHAR                                "treatAsChar"
#define    JIN_TABLE_POS                                        "pos"
#define    JIN_TABLE_TEXTPOS                                    "textpos"
#define    JIN_TABLE_HTYPE                                        "horizontal-type"
#define    JIN_TABLE_HDIRECTION                                "horizontal-direction"
#define    JIN_TABLE_HVALUE                                    "horizontal-value"
#define    JIN_TABLE_VTYPE                                        "vertical-type"
#define    JIN_TABLE_VDIRECTION                                "vertical-direction"
#define    JIN_TABLE_VVALUE                                    "vertical-value"
#define    JIN_TABLE_SOFLOWWITHTEXT                            "soflowwithtext"
#define    JIN_TABLE_SOALLOWOVERLAP                            "soallowoverlap"
#define    JIN_TABLE_SOHOLDANCHOROBJ                            "soholdanchorobj"
#define    JIN_TABLE_PARALLEL                                    "parallel"
#define    JIN_TABLE_ROTATION                                    "rotation"
#define    JIN_TABLE_GRADIENT_H                                "gradientH"
#define    JIN_TABLE_GRADIENT_V                                "gradientV"
#define    JIN_TABLE_NUMVERTYPE                                "numbertype"
#define    JIN_TABLE_OBJPROTECT                                "objprotect"
#define    JIN_TABLE_MARGIN_LEFT                                "margin-left"
#define    JIN_TABLE_MARGIN_RIGHT                                "margin-right"
#define    JIN_TABLE_MARGIN_TOP                                "margin-top"
#define    JIN_TABLE_MARGIN_BOTTOM                                "margin-bottom"
#define    JIN_TABLE_CAPTION_POSITION                            "caption-position"
#define    JIN_TABLE_CAPTION_SIZE                                "caption-size"
#define    JIN_TABLE_CAPTION_SPACING                            "caption-spacing"
#define    JIN_TABLE_CAPTION_SOCAPFULLSIZE                        "caption-socapfullsize"
#define    JIN_TABLE_CAPTION_LINEWRAP                            "caption-linewrap"
#define    JIN_TABLE_BORDER                                    "border"
#define    JIN_TABLE_BORDER_POSITION                            "position"
#define    JIN_TABLE_BORDER_TYPE                                "bordertype"
#define    JIN_TABLE_BORDER_SIZE                                "size"
#define    JIN_TABLE_BORDER_COLOR                                "color"
#define    JIN_TABLE_BORDER_CELLSPACING                        "border-cellspacing"
#define    JIN_TABLE_BGFILL_TYPE                                "bgfill-type"
#define    JIN_TABLE_BGFILL_FACECOLOR                            "bgfill-facecolor"
#define    JIN_TABLE_BGFILL_PATTONCOLOR                        "bgfill-pattoncolor"
#define    JIN_TABLE_BGFILL_PATTONTYPE                            "bgfill-pattontype"
#define    JIN_TABLE_BGGRADATION_STARTCOLOR                    "bggradation-startcolor"
#define    JIN_TABLE_BGGRADATION_ENDCOLOR                        "bggradation-endcolor"
#define    JIN_TABLE_BGGRADATION_TYPE                            "bggradation-type"
#define    JIN_TABLE_BGGRADATION_WIDTHCENTER                    "bggradation-widthcenter"
#define    JIN_TABLE_BGGRADATION_HEIGHTCENTER                    "bggradation-heightcenter"
#define    JIN_TABLE_BGGRADATION_GRADATIONANGLE                "bggradation-gradationangle"
#define    JIN_TABLE_BGGRADATION_BLURLEVEL                        "bggradation-blurlevel"
#define    JIN_TABLE_BGGRADATION_BLURCENTER                    "bggradation-blurcenter"
#define    JIN_TABLE_PICTURE_FILE                                "picture-file"
#define    JIN_TABLE_PICTURE_INCLUDE                            "picture-include"
#define    JIN_TABLE_PICTUREFILL_TYPE                            "picturefill-type"
#define    JIN_TABLE_PICTUREFILL_VALUE                            "picturefill-value"
#define    JIN_TABLE_EFFECT_TYPE                                "effect-type"
#define    JIN_TABLE_EFFECT_VALUE                                "effect-value"
#define    JIN_TABLE_WATERMARK                                    "watermark"
#define    JIN_TABLE_TABLE_IN_TABLE                            "table-in-table"


#define    JID_TABLE                                            3000
#define    JID_TABLE_SIZEWIDTH                                    JID_TABLE+1
#define    JID_TABLE_SIZEHEIGHT                                JID_TABLE+2
#define    JID_TABLE_SIZEFIXED                                    JID_TABLE+3
#define    JID_TABLE_TREATASCHAR                                JID_TABLE+4
#define    JID_TABLE_POS                                        JID_TABLE+5
#define    JID_TABLE_TEXTPOS                                    JID_TABLE+6
#define    JID_TABLE_HTYPE                                        JID_TABLE+7
#define    JID_TABLE_HDIRECTION                                JID_TABLE+8
#define    JID_TABLE_HVALUE                                    JID_TABLE+9
#define    JID_TABLE_VTYPE                                        JID_TABLE+10
#define    JID_TABLE_VDIRECTION                                JID_TABLE+11
#define    JID_TABLE_VVALUE                                    JID_TABLE+12
#define    JID_TABLE_SOFLOWWITHTEXT                            JID_TABLE+13
#define    JID_TABLE_SOALLOWOVERLAP                            JID_TABLE+14
#define    JID_TABLE_SOHOLDANCHOROBJ                            JID_TABLE+15
#define    JID_TABLE_PARALLEL                                    JID_TABLE+16
#define    JID_TABLE_ROTATION                                    JID_TABLE+17
#define    JID_TABLE_GRADIENT_H                                JID_TABLE+18
#define    JID_TABLE_GRADIENT_V                                JID_TABLE+19
#define    JID_TABLE_NUMVERTYPE                                JID_TABLE+20
#define    JID_TABLE_OBJPROTECT                                JID_TABLE+21
#define    JID_TABLE_MARGIN_LEFT                                JID_TABLE+22
#define    JID_TABLE_MARGIN_RIGHT                                JID_TABLE+23
#define    JID_TABLE_MARGIN_TOP                                JID_TABLE+24
#define    JID_TABLE_MARGIN_BOTTOM                                JID_TABLE+25
#define    JID_TABLE_CAPTION_POSITION                            JID_TABLE+26
#define    JID_TABLE_CAPTION_SIZE                                JID_TABLE+27
#define    JID_TABLE_CAPTION_SPACING                            JID_TABLE+28
#define    JID_TABLE_CAPTION_SOCAPFULLSIZE                        JID_TABLE+29
#define    JID_TABLE_CAPTION_LINEWRAP                            JID_TABLE+30
#define    JID_TABLE_BORDER                                    JID_TABLE+31
#define    JID_TABLE_BORDER_POSITION                            JID_TABLE+32
#define    JID_TABLE_BORDER_TYPE                                JID_TABLE+33
#define    JID_TABLE_BORDER_SIZE                                JID_TABLE+34
#define    JID_TABLE_BORDER_COLOR                                JID_TABLE+35
#define    JID_TABLE_BORDER_CELLSPACING                        JID_TABLE+36
#define    JID_TABLE_BGFILL_TYPE                                JID_TABLE+37
#define    JID_TABLE_BGFILL_FACECOLOR                            JID_TABLE+38
#define    JID_TABLE_BGFILL_PATTONCOLOR                        JID_TABLE+39
#define    JID_TABLE_BGFILL_PATTONTYPE                            JID_TABLE+40
#define    JID_TABLE_BGGRADATION_STARTCOLOR                    JID_TABLE+41
#define    JID_TABLE_BGGRADATION_ENDCOLOR                        JID_TABLE+42
#define    JID_TABLE_BGGRADATION_TYPE                            JID_TABLE+43
#define    JID_TABLE_BGGRADATION_WIDTHCENTER                    JID_TABLE+44
#define    JID_TABLE_BGGRADATION_HEIGHTCENTER                    JID_TABLE+45
#define    JID_TABLE_BGGRADATION_GRADATIONANGLE                JID_TABLE+46
#define    JID_TABLE_BGGRADATION_BLURLEVEL                        JID_TABLE+47
#define    JID_TABLE_BGGRADATION_BLURCENTER                    JID_TABLE+48
#define    JID_TABLE_PICTURE_FILE                                JID_TABLE+49
#define    JID_TABLE_PICTURE_INCLUDE                            JID_TABLE+50
#define    JID_TABLE_PICTUREFILL_TYPE                            JID_TABLE+51
#define    JID_TABLE_PICTUREFILL_VALUE                            JID_TABLE+52
#define    JID_TABLE_EFFECT_TYPE                                JID_TABLE+53
#define    JID_TABLE_EFFECT_VALUE                                JID_TABLE+54
#define    JID_TABLE_WATERMARK                                    JID_TABLE+55
#define    JID_TABLE_TABLE_IN_TABLE                            JID_TABLE+56
#define    JID_TABLE_END                                        JID_TABLE+999


//////////////////////////////////////////////////////////
// 특수문자 : JID_SPECIALCHARACTER

#define    JIN_SPECIALCHARACTER                                "specialcharacter"
#define    JIN_SPECIALCHARACTER_MINIMUM                        "minimum"
#define    JIN_SPECIALCHARACTER_MAXIMUM                        "maximum"

#define    JID_SPECIALCHARACTER                                3100
#define    JID_SPECIALCHARACTER_MINIMUM                        JID_SPECIALCHARACTER+1
#define    JID_SPECIALCHARACTER_MAXIMUM                        JID_SPECIALCHARACTER+2
#define    JID_SPECIALCHARACTER_END                            JID_SPECIALCHARACTER+99



//////////////////////////////////////////////////////////
// 개요 번호 모양 : JID_OUTLINESHAPE

#define    JIN_OUTLINESHAPE                                    "outlineshape"
#define    JIN_OUTLINESHAPE_TYPE                                "type"
#define    JIN_OUTLINESHAPE_STARTNUMBER                        "startnumber"
#define    JIN_OUTLINESHAPE_VALUE                                "value"
#define    JIN_OUTLINESHAPE_LEVELTYPE                            "leveltype"
#define    JIN_OUTLINESHAPE_LEVELTYPE_LEVEL                    "level"
#define    JIN_OUTLINESHAPE_LEVELTYPE_NUMBERTYPE                "numbertype"
#define    JIN_OUTLINESHAPE_LEVELTYPE_NUMBERSHAPE                "numbershape"

#define    JID_OUTLINESHAPE                                    3200
#define    JID_OUTLINESHAPE_TYPE                                JID_OUTLINESHAPE+1
#define    JID_OUTLINESHAPE_STARTNUMBER                        JID_OUTLINESHAPE+2
#define    JID_OUTLINESHAPE_VALUE                                JID_OUTLINESHAPE+3
#define    JID_OUTLINESHAPE_LEVELTYPE                            JID_OUTLINESHAPE+4
#define    JID_OUTLINESHAPE_LEVELTYPE_LEVEL                    JID_OUTLINESHAPE+5
#define    JID_OUTLINESHAPE_LEVELTYPE_NUMBERTYPE                JID_OUTLINESHAPE+6
#define    JID_OUTLINESHAPE_LEVELTYPE_NUMBERSHAPE                JID_OUTLINESHAPE+7
#define    JID_OUTLINESHAPE_END                                JID_OUTLINESHAPE+99



//////////////////////////////////////////////////////////
// 글머리표 : JID_BULLET

#define    JIN_BULLET                                            "bullet"
#define    JIN_BULLET_TYPE                                        "type"
#define    JIN_BULLET_CHECKTYPE                                "checktype"
#define    JIN_BULLET_CODE                                        "code"
#define    JIN_BULLET_SHAPES                                    "bulletshapes"

#define    JID_BULLET                                            3300
#define    JID_BULLET_TYPE                                        JID_BULLET+1
#define    JID_BULLET_CHECKTYPE                                JID_BULLET+2
#define    JID_BULLET_CODE                                        JID_BULLET+3
#define JID_BULLET_SHAPES                                    JID_BULLET+4
#define    JID_BULLET_END                                        JID_BULLET+99



//////////////////////////////////////////////////////////
// 문단 번호 : JID_PARANUMBULLET

#define    JIN_PARANUMBULLET                                    "paranumbullet"
#define    JIN_PARANUMBULLET_TYPE                                "type"
#define    JIN_PARANUMBULLET_STARTNUMBER                        "startnumber"
#define    JIN_PARANUMBULLET_VALUE                                "value"
#define    JIN_PARANUMBULLET_LEVELTYPE                            "leveltype"
#define    JIN_PARANUMBULLET_LEVELTYPE_LEVEL                    "level"
#define    JIN_PARANUMBULLET_LEVELTYPE_NUMBERTYPE                "numbertype"
#define    JIN_PARANUMBULLET_LEVELTYPE_NUMBERSHAPE                "numbershape"

#define    JID_PARANUMBULLET                                    3400
#define    JID_PARANUMBULLET_TYPE                                JID_PARANUMBULLET+1
#define    JID_PARANUMBULLET_STARTNUMBER                        JID_PARANUMBULLET+2
#define    JID_PARANUMBULLET_VALUE                                JID_PARANUMBULLET+3
#define    JID_PARANUMBULLET_LEVELTYPE                            JID_PARANUMBULLET+4
#define    JID_PARANUMBULLET_LEVELTYPE_LEVEL                    JID_PARANUMBULLET+5
#define    JID_PARANUMBULLET_LEVELTYPE_NUMBERTYPE                JID_PARANUMBULLET+6
#define    JID_PARANUMBULLET_LEVELTYPE_NUMBERSHAPE                JID_PARANUMBULLET+7
#define    JID_PARANUMBULLET_END                                JID_PARANUMBULLET+99



//////////////////////////////////////////////////////////
// 스타일 : JID_STYLE

#define    JIN_STYLE                                            "style"
#define    JIN_STYLE_TYPE                                        "type"
#define    JIN_STYLE_PERMISSION                                "permission"

#define    JID_STYLE                                            3500
#define    JID_STYLE_TYPE                                        JID_STYLE+1
#define    JID_STYLE_PERMISSION                                JID_STYLE+2
#define    JID_STYLE_END                                        JID_STYLE+99



//////////////////////////////////////////////////////////
// 페이지 : JID_PAGE

#define    JIN_PAGE                                            "page"
#define    JIN_PAGE_TYPE                                        "type"
#define    JIN_PAGE_WIDTH                                        "width"
#define    JIN_PAGE_HEIGHT                                        "height"
#define    JIN_PAGE_DIRECTION                                    "direction"
#define    JIN_PAGE_BOOKBINDING                                "bookbinding"
#define    JIN_PAGE_MARGIN_TOP                                    "margin-top"
#define    JIN_PAGE_MARGIN_HEADER                                "margin-header"
#define    JIN_PAGE_MARGIN_LEFT                                "margin-left"
#define    JIN_PAGE_MARGIN_BOOKBINDING                            "margin-bookbinding"
#define    JIN_PAGE_MARGIN_RIGHT                                "margin-right"
#define    JIN_PAGE_MARGIN_FOOT                                "margin-foot"
#define    JIN_PAGE_MARGIN_BOTTOM                                "margin-bottom"
#define    JIN_PAGE_APPLYRANGE                                    "applyrange"
#define    JIN_PAGE_APPLYNEWDOC                                "applynewdoc"
#define    JIN_PAGE_GRID_TYPE                                    "grid-type"
#define    JIN_PAGE_GRID_LINECOUNT                                "grid-linecount" 
#define    JIN_PAGE_GRID_LINESPACE                                "grid-linespace" 
#define    JIN_PAGE_GRID_GRADATION                                "grid-gradation" 
#define    JIN_PAGE_GRID_CHARSPACE                                "grid-charspace"
#define    JIN_PAGE_GRID_ORTHOGRAPHY                            "grid-orthography"
#define    JIN_PAGE_GRID_APPLYRANGE                            "grid-applyrange"

#define    JID_PAGE                                            4000
#define    JID_PAGE_TYPE                                        JID_PAGE+1
#define    JID_PAGE_WIDTH                                        JID_PAGE+2
#define    JID_PAGE_HEIGHT                                        JID_PAGE+3
#define    JID_PAGE_DIRECTION                                    JID_PAGE+4
#define    JID_PAGE_BOOKBINDING                                JID_PAGE+5
#define    JID_PAGE_MARGIN_TOP                                    JID_PAGE+6
#define    JID_PAGE_MARGIN_HEADER                                JID_PAGE+7
#define    JID_PAGE_MARGIN_LEFT                                JID_PAGE+8
#define    JID_PAGE_MARGIN_BOOKBINDING                            JID_PAGE+9
#define    JID_PAGE_MARGIN_RIGHT                                JID_PAGE+10
#define    JID_PAGE_MARGIN_FOOT                                JID_PAGE+11
#define    JID_PAGE_MARGIN_BOTTOM                                JID_PAGE+12
#define    JID_PAGE_APPLYRANGE                                    JID_PAGE+13
#define    JID_PAGE_APPLYNEWDOC                                JID_PAGE+14
#define    JID_PAGE_GRID_TYPE                                    JID_PAGE+15
#define    JID_PAGE_GRID_LINECOUNT                                JID_PAGE+16
#define    JID_PAGE_GRID_LINESPACE                                JID_PAGE+17
#define    JID_PAGE_GRID_GRADATION                                JID_PAGE+18
#define    JID_PAGE_GRID_CHARSPACE                                JID_PAGE+19
#define    JID_PAGE_GRID_ORTHOGRAPHY                            JID_PAGE+20
#define    JID_PAGE_GRID_APPLYRANGE                            JID_PAGE+21
#define    JID_PAGE_END                                        JID_PAGE+999


//////////////////////////////////////////////////////////
// 문서 정보  : JID_DOCSUMMARYINFO

#define    JIN_DOCSUMMARYINFO                            "DocSummaryInfo"

#define    JID_DOCSUMMARYINFO                            5000



//////////////////////////////////////////////////////////
// 각주 : JID_FOOTNOTE

#define    JIN_FOOTNOTE                                "footnote"

#define    JID_FOOTNOTE                                6000



//////////////////////////////////////////////////////////
// 미주 : JID_ENDNOTE

#define    JIN_ENDNOTE                                    "endnote"

#define    JID_ENDNOTE                                    6100



//////////////////////////////////////////////////////////
// 메모 : JID_MEMO

#define    JIN_MEMO                                    "memo"

#define    JID_MEMO                                    6200



//////////////////////////////////////////////////////////
// 차트 : JID_CHART

#define    JIN_CHART                                    "chart"

#define    JID_CHART                                    6300



//////////////////////////////////////////////////////////
// 글맵시 : JID_WORDART

#define    JIN_WORDART                                    "wordart"

#define    JID_WORDART                                    6400



//////////////////////////////////////////////////////////
// 수식 : JID_FORMULA

#define    JIN_FORMULA                                    "formula"

#define    JID_FORMULA                                    6500



//////////////////////////////////////////////////////////
// OLE : JID_OLE

#define    JIN_OLE                                        "ole"

#define    JID_OLE                                        6600



//////////////////////////////////////////////////////////
// 양식 개체 : JID_FORMOBJECT

#define    JIN_FORMOBJECT                                "formobject"

#define    JID_FORMOBJECT                                6700



//////////////////////////////////////////////////////////
// 책갈피 : JID_BOOKMARK

#define    JIN_BOOKMARK                                "bookmark"

#define    JID_BOOKMARK                                6800



//////////////////////////////////////////////////////////
// 하이퍼링크 : JID_HYPERLINK

#define    JIN_HYPERLINK                                "hyperlink"
#define JIN_HYPERLINK_PERMISSION                    "permission"

#define    JID_HYPERLINK                                6900
#define JID_HYPERLINK_PERMISSION                    JID_HYPERLINK+1
#define JID_HYPERLINK_END                            JID_HYPERLINK+99



//////////////////////////////////////////////////////////
// 메크로 : JID_MECRO

#define    JIN_MACRO                                    "macro"
#define JIN_MACRO_PERMISSION                        "permission"

#define    JID_MACRO                                    7000
#define JID_MACRO_PERMISSION                        JID_MACRO+1
#define JID_MACRO_END                                JID_MACRO+99