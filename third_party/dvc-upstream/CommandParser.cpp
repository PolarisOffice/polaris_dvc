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

#include "pch.h"

#include <iostream>

#include <locale>
#include <codecvt>
#include <string>

#include "CommandParser.h"

#include "Util.h"

CommandParser& getCommandParser()
{
    static CommandParser parser;
    return parser;
}


CommandParser::CommandParser()
{
    m_outputOption = DVC::DVCOutputOption::Default;
}

CommandParser::~CommandParser()
{
}

bool CommandParser::commandParsing(int argc, DVC_CHAR* argv[])
{
    m_message.clear();
    m_optionValue.clear();

    // Preset
    presetOption();

    // Parsing
    if (argc <= 1) {
        // command 가 비워져 있으면 Help 메세지를 설정한다.
        setErrorMessage(ErrorMsgCode::Help);

        if (m_optionValue.empty() == true) {
            // 미리 설정된 값이 있다면 false 가 아니다.
            return true;
        }

        return false;
    }

    for (int argIndex = 1; argIndex < argc; argIndex++) {
        // Convert
        DVC_STRING argValue;
#ifdef OS_LINUX
        argValue.assign(argv[argIndex]);
#endif
#ifdef WINDOWS
        {
            argValue.assign(argv[argIndex]);
        }
#endif

        // 어떠한 옵션을 사용하더라도 2글자 이상이다.
        if (argValue.length() <= 1) {
            setErrorMessage(ErrorMsgCode::InvalidOption, &argValue);
            return m_message.c_str();
        }

        // option
        if (argValue[0] == '-') {
            // short option
            if (argValue[1] != '-') {
                parsingShortOption(argValue);
            }
            // long option
            else {
                parsingLongOption(argValue);
            } // else if
        } // if

        // file path ?
        else {
            // argValue 를 절대 경로로 변경시킨다.
            DVC_STRING& filepath = (m_dvcFilepath.empty() == true) ? m_dvcFilepath : m_targetFilepath;
            bool isValid = Util::getAbsolutePath(argValue, filepath);
            if (isValid == false) {
                setErrorMessage(ErrorMsgCode::NoExistFile, &argValue);
            } // if
        } /// else

        // Message가 있으면 중지한다.
        if (m_message.empty() == false) {
            break;
        }
    } // for

    return m_message.empty();
}


bool CommandParser::isEnableOption(DVC::Options option)
{
    auto findItr = m_optionValue.find(option);
    return (findItr != m_optionValue.end()) ? true : false;
}

const DVC_STRING* CommandParser::getOptionValue(DVC::Options option)
{
    auto findItr = m_optionValue.find(option);
    if (findItr != m_optionValue.end()) {
        return &(*findItr).second;
    }

    return NULL;
}

void CommandParser::setOption(DVC::Options option)
{
    DVC_STRING dummy;

    setOption(option, dummy);
}

void CommandParser::setOption(DVC::Options option, const DVC_STRING& value /*= NULL*/)
{
    // Option balance
    switch (option) {
    case DVC::Options::LevelAll:
        m_optionValue.erase(DVC::Options::LevelAll);
        break;
    case DVC::Options::LevelSimple:
        m_optionValue.erase(DVC::Options::LevelSimple);
        break;
    }

    // Setup
    auto findItr = m_optionValue.find(option);
    if (findItr == m_optionValue.end()) {
        // Insert
        m_optionValue.insert(std::pair<DVC::Options, DVC_STRING>(option, value));
    }
    else {
        // Update
        m_optionValue.at(option) = value;
    }
}

void CommandParser::SetOutputOption(DVC::DVCOutputOption option)
{
    m_outputOption = option;
}

DVC::DVCOutputOption CommandParser::GetOutputOption()
{
    return m_outputOption;
}

const DVC_STRING&    CommandParser::getDVCFilepath()
{
    return m_dvcFilepath;
}

const DVC_STRING&    CommandParser::getTargetFilepath()
{
    return m_targetFilepath;
}

void CommandParser::setDVCFilepath(DVC_LPCSTR filepath)
{
    m_dvcFilepath.assign(filepath);
}

void CommandParser::setTargetFilepath(DVC_LPCSTR filepath)
{
    m_targetFilepath.assign(filepath);
}

const DVC_STRING& CommandParser::getMessage()
{
    return m_message;
}

void CommandParser::presetOption()
{
    // Default value
    setOption(DVC::Options::FormatJSON);
    setOption(DVC::Options::OutputConsole);
    setOption(DVC::Options::LevelAll);
    SetOutputOption(DVC::DVCOutputOption::Default);
}

bool CommandParser::parsingShortOption(const DVC_STRING& argValue)
{
    for (size_t index = 1; index < argValue.length(); index++) {
        switch (argValue.at(index)) {
            // JSON
        case 'j':
            setOption(DVC::Options::FormatJSON);
            break;

            // XML
        case 'x':
            setErrorMessage(ErrorMsgCode::NotYet, &(L"-" + argValue.substr(index, 1)));
            index = argValue.length() + 1;    // Exit for
            break;

            // Output to console
        case 'c':
            setOption(DVC::Options::OutputConsole);
            break;

            // check level
        case 's':
            setOption(DVC::Options::LevelSimple);
            break;

        case 'a':
            setOption(DVC::Options::LevelAll);
            break;

            // Help
        case 'H':
        case 'h':
            setErrorMessage(ErrorMsgCode::Help);
            index = argValue.length() + 1;    // Exit for
            break;

            // Version
        case 'v':
            setErrorMessage(ErrorMsgCode::Version);
            index = argValue.length() + 1;    // Exit for
            break;

        case 'd':
            SetOutputOption(DVC::DVCOutputOption::Default);
            break;

        case 'o':
            SetOutputOption(DVC::DVCOutputOption::AllOption);
            break;

        case 't':
            SetOutputOption(DVC::DVCOutputOption::Table);
            break;

        case 'i':
            SetOutputOption(DVC::DVCOutputOption::TableDetail);
            break;

        case 'p':
            SetOutputOption(DVC::DVCOutputOption::Shape);
            break;

        case 'y':
            SetOutputOption(DVC::DVCOutputOption::Style);
            break;

        case 'k':
            SetOutputOption(DVC::DVCOutputOption::Hyperlink);
            break;

        default:
            setErrorMessage(ErrorMsgCode::InvalidOption, &(L"-" + argValue.substr(index, 1)));
            index = argValue.length() + 1;    // Exit for
            break;
        }
    }

    return !m_message.empty();
}


void CommandParser::parsingLongOption(const DVC_STRING& argValue)
{
    // Key & Value
    DVC_STRING key;
    DVC_STRING value;

    size_t equalIndex = argValue.find(L"=");
    if (equalIndex != std::wstring::npos) {
        key.assign(argValue.substr(2, equalIndex - 2));
        value.assign(argValue.substr(equalIndex + 1));
    }
    else {
        key.assign(argValue.substr(2));
    }

    // Format
    if (key.compare(DVCString::Format) == 0) {
        if (value.compare(DVCString::Json) == 0) {
            setOption(DVC::Options::FormatJSON);
        }
        else if (value.compare(DVCString::Xml) == 0) {
            setOption(DVC::Options::FormatXML);
        }
        else if (value.compare(DVCString::Hwpx) == 0) {
            setErrorMessage(ErrorMsgCode::NotYet, &argValue);
            return;
        }
        else if (value.compare(DVCString::Text) == 0) {
            setErrorMessage(ErrorMsgCode::NotYet, &argValue);
            return;
        }
        else {
            setErrorMessage(ErrorMsgCode::InvalidOption, &argValue);
        }
    }

    // Console
    else if (key.compare(DVCString::Console) == 0) {
        setOption(DVC::Options::OutputConsole);
    }

    // File
    else if (key.compare(DVCString::File) == 0) {
        setOption(DVC::Options::OutputFile, value);
    }

    // Check level
    else if (key.compare(DVCString::Simple) == 0) {
        setOption(DVC::Options::LevelSimple);
    }
    else if (value.compare(DVCString::All) == 0) {
        setOption(DVC::Options::LevelAll);
    }

    // Output option
    else if (key.compare(DVCString::OutputDefault) == 0) {
        SetOutputOption(DVC::DVCOutputOption::Default);
    }
    else if (key.compare(DVCString::OutputAllOption) == 0) {
        SetOutputOption(DVC::DVCOutputOption::AllOption);
    }
    else if (key.compare(DVCString::OutputTable) == 0) {
        SetOutputOption(DVC::DVCOutputOption::Table);
    }
    else if (key.compare(DVCString::OutputTableDetail) == 0) {
        SetOutputOption(DVC::DVCOutputOption::TableDetail);
    }
    else if (key.compare(DVCString::OutputShape) == 0) {
        SetOutputOption(DVC::DVCOutputOption::Shape);
    }
    else if (key.compare(DVCString::OutputStyle) == 0) {
        SetOutputOption(DVC::DVCOutputOption::Style);
    }
    else if (key.compare(DVCString::OutputHyperlink) == 0) {
        SetOutputOption(DVC::DVCOutputOption::Hyperlink);
    }
    
    // Help
    else if (key.compare(L"help") == 0) {
        setErrorMessage(ErrorMsgCode::Help);
    }

    // Version
    else if (key.compare(L"version") == 0) {
        setErrorMessage(ErrorMsgCode::Version);
    }

    // Unknown
    else {
        setErrorMessage(ErrorMsgCode::InvalidOption, &argValue);
    }
}

void CommandParser::setErrorMessage(ErrorMsgCode msgCode, const DVC_STRING* replaceValue/* = NULL*/)
{
    DVC_STRING errMessage;

    switch (msgCode) {
    case ErrorMsgCode::Help:
        errMessage.assign(StrLiteral(
            "Usage: hwpxdiff [OPTION]... [Previous HWPX file] [Modified HWPX file]\r\n\
\r\n\
== Specifies the form for the document comparison result == \r\n\
   -j    --format=json           JSON format. (Default value)\r\n\
\r\n\
== How to output results == \r\n\
   -c    --console               Output to console. (Default value)\r\n\
         --file=[filepath]       Output to file.\r\n\
                                 Existing files, if any, are replaced.\r\n\
\r\n\
== Level of comparison == \r\n\
   -p    --level=paragraph       Compare paragraph content(Default value).\r\n\
                                 It cannot be used with \"--level=dividePara\".\r\n\
         --level=dividePara      Compare paragraphs by dividing them into units.\r\n\
                                 It cannot be used with \"--level = paragraph\".\r\n\
   -a    --level=attribute       Compare attribute values.\r\n\
\r\n\
== Help == \r\n\
   -h    --help                  Display this help and exit\r\n\
   -v    --version               Output version information and exit"));
        break;

    case ErrorMsgCode::Version:
        errMessage.assign(StrLiteral("Version : 0.01.2"));
        break;

    case ErrorMsgCode::NoExistFile:
        errMessage.assign(StrLiteral("No exist file : %s"));
        break;

    case ErrorMsgCode::InvalidOption:
        errMessage.assign(StrLiteral("Invalid option '%s'\r\nTry '--help' for more information."));
        break;

    case ErrorMsgCode::NotYet:
        errMessage.assign(StrLiteral("Option not yet available '%s'\r\nTry '--help' for more information."));
        break;
    }

    setMessage(errMessage, replaceValue);
}

void CommandParser::setMessage(const DVC_STRING& setMessage, const DVC_STRING* replaceValue/* = NULL*/)
{
    m_message.assign(setMessage);
    if (replaceValue != NULL) {
        size_t findIndx = m_message.find(DVCString::CmdParserErrPlaceHolder);
        if (findIndx != std::wstring::npos) {
            m_message.erase(findIndx, 2);
            m_message.insert(findIndx, *replaceValue);
        }
    }
}