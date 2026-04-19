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
#ifndef __DVC_EXPORT_H__
#define __DVC_EXPORT_H__

#ifdef WINDOWS
#ifdef DVC_EXPORTS
#define DVC_DLL_DECLSPEC    __declspec(dllexport)
#else
#define DVC_DLL_DECLSPEC    __declspec(dllimport)
#endif
#else
#define DVC_DLL_DECLSPEC 
#endif // WINDOWS

#include "./ExportInterface.h"


namespace DVC {
    // extern "C" DVC_DLL_DECLSPEC bool testHello(const char* beforeTextPath, const char* afterTextPath);


    extern "C" DVC_DLL_DECLSPEC IDVC* createDVC(/*std::wstring dvcFile, std::wstring hwpxFile, bool printOutput*/);
    extern "C" DVC_DLL_DECLSPEC void deleteDVC(IDVC* object);
}

#endif // __DVC_EXPORT_H__