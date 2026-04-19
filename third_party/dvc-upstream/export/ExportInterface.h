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

#ifndef __DVC_EXPORT_INTERFACE_H__
#define __DVC_EXPORT_INTERFACE_H__

#include "../DVCDefine.h"
#include <vector>


namespace DVC {

    enum DVCOutputOption {
        Default = 0,
        AllOption,
        Table,
        TableDetail,
        Shape,
        Style,
        Hyperlink
    };

    enum DVCState {
        oRemove = -1,
        oModify,
        oAdd,
    };
    
    enum Options {
        Help = 0x00001,    // Inclusive
        Version = 0x00002,

        Format = 0x00100,    // Exclusive
        FormatJSON,    // Exclusive
        FormatXML,
        FormatHwpx,
        FormatText,

        Output = 0x00200,    // Exclusive
        OutputConsole,
        OutputFile,

        Level = 0x00300,    // Exclusive
        LevelAll,
        LevelSimple,
        LevelAttribute = 0x00310,    // Inclusive
    };

    __interface IDVC
    {
    public:
        virtual DVC_LPCSTR setCommand(int argc, DVC_CHAR* argv[]) = 0;
        virtual void setOption(Options option, DVC_LPCSTR value = NULL) = 0;
        virtual void setJsonFilepath(DVC_LPCSTR filepath) = 0;
        virtual void setTargetFilepath(DVC_LPCSTR filepath) = 0;
        virtual bool doValidationCheck() = 0;
        virtual bool output() = 0;
        virtual bool output(Options outputOption, DVC_LPCSTR filePath = NULL) = 0;
    };


    __interface IDVCErrInfo {
    public:
        virtual UINT getCharPrIDRef();    
        virtual UINT getParaPrIDRef();
        virtual std::wstring getText();
        virtual UINT getPageNo();
        virtual UINT getLineNo();
        virtual UINT getErrorCode();

        virtual bool getIsInTable();
        virtual bool getIsInTableInTable();
        virtual bool getIsInShape();
        virtual UINT getTableID();
             
        virtual UINT getTableRow();
        virtual UINT getTableCol();
             
        virtual bool getUseHyperlink();
        virtual bool getUseStyle();
             
        virtual std::wstring getErrorString();        
    };

    enum OutputType
    {
        JSON,
        //XML,
        //Text,
    };

    __interface IDVCOutput
    {
    public:
        virtual OutputType getType() = 0;
        virtual void appendResult(IDVCErrInfo* result) = 0;
        virtual void writeToFile(const DVC_STRING& filename, DVCOutputOption option) = 0;
        virtual void writeToConsole(DVCOutputOption option) = 0;
    };


}

#endif // __DVC_EXPORT_INTERFACE_H__
