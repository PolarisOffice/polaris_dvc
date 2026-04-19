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

#ifndef __DVC_COMMAND_PARSER__
#define __DVC_COMMAND_PARSER__

#include "export\ExportInterface.h"
#include "DVCDefine.h"

class CommandParser
{
public:
    CommandParser();
    ~CommandParser();

public:
    bool commandParsing(int argc, DVC_CHAR* argv[]);

    bool isEnableOption(DVC::Options option);
    const DVC_STRING* getOptionValue(DVC::Options option);
    void setOption(DVC::Options option);
    void setOption(DVC::Options option, const DVC_STRING& value);
    void SetOutputOption(DVC::DVCOutputOption option);
    DVC::DVCOutputOption GetOutputOption();

    const DVC_STRING&    getDVCFilepath();
    const DVC_STRING&    getTargetFilepath();
    void setDVCFilepath(DVC_LPCSTR filepath);
    void setTargetFilepath(DVC_LPCSTR filepath);

    const DVC_STRING& getMessage();


private:
    enum ErrorMsgCode {
        Help = 1,
        Version,
        NoExistFile,
        InvalidOption,
        NotYet,
    };

    DVC_STRING    m_message;
    DVC_STRING    m_dvcFilepath;
    DVC_STRING    m_targetFilepath;
    DVC::DVCOutputOption m_outputOption;

    std::map<DVC::Options, DVC_STRING> m_optionValue;

    void presetOption();

    bool parsingShortOption(const DVC_STRING& argValue);
    void parsingLongOption(const DVC_STRING& argValue);

    void setErrorMessage(ErrorMsgCode msgCode, const DVC_STRING* replaceValue = NULL);
    void setMessage(const DVC_STRING& message, const DVC_STRING* replaceValue = NULL);
};

CommandParser& getCommandParser();

#endif // __DVC_COMMAND_PARSER__