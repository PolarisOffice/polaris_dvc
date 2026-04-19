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

#include "DVCInterface.h"

#include "..//Checker.h"


class DVCModule : public DVC::IDVC
{
public:
    DVCModule(/*DVC_LPCSTR dvcFile, DVC_LPCSTR hwpxFile, bool printOutput = false*/);
    ~DVCModule();

public:

    virtual DVC_LPCSTR setCommand(int argc, DVC_CHAR* argv[]);
    virtual void setOption(DVC::Options option, DVC_LPCSTR value = NULL);
    virtual void setJsonFilepath(DVC_LPCSTR filepath);
    virtual void setTargetFilepath(DVC_LPCSTR filepath);
    virtual bool doValidationCheck();
    virtual bool output();
    virtual bool output(DVC::Options outputOption, DVC_LPCSTR filePath = NULL);

private:
    std::wstring        m_dvcFile;
    std::wstring        m_hwpxFile;

    Checker*            m_checker;

    bool                m_printOutput;
    bool                m_checkAll;

    DVC::IDVCOutput*    m_dvcOutput;
};

