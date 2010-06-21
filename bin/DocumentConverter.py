#!/usr/bin/python
#
# PyODConverter (Python OpenDocument Converter) v1.1 - 2009-11-14
# Modifications by Mikko Rantalainen <mikko.rantalainen@peda.net>
#
# This script converts a document from one office format to another by
# connecting to an OpenOffice.org instance via Python-UNO bridge.
#
# Copyright (C) 2008-2009 Mirko Nasato <mirko@artofsolving.com>
# Licensed under the GNU LGPL v2.1 - http://www.gnu.org/licenses/lgpl-2.1.html
# - or any later version.
#
# See also:
# http://www.artofsolving.com/opensource/pyodconverter
# http://www.linuxjournal.com/content/starting-stopping-and-connecting-openoffice-python
#

DEFAULT_OPENOFFICE_PORT = 8100

import sys
import os
import time

# Find OpenOffice.
_oopaths=(
        ('/usr/lib/openoffice/program',   '/usr/lib/openoffice/program'),
        ('/usr/lib64/ooo-2.0/program',   '/usr/lib64/ooo-2.0/program'),
        ('/opt/openoffice.org3/program', '/opt/openoffice.org/basis3.0/program'),
     )

for p in _oopaths:
    if os.path.exists(p[0]):
        OPENOFFICE_PATH    = p[0]
        OPENOFFICE_BIN     = os.path.join(OPENOFFICE_PATH, 'soffice')
        OPENOFFICE_LIBPATH = p[1]

        # Add to path so we can find uno.
        if sys.path.count(OPENOFFICE_LIBPATH) == 0:
            sys.path.insert(0, OPENOFFICE_LIBPATH)
        break

import uno
from os.path import abspath, isfile, splitext
from com.sun.star.beans import PropertyValue
from com.sun.star.task import ErrorCodeIOException
from com.sun.star.connection import NoConnectException

FAMILY_TEXT = "Text"
FAMILY_WEB = "Web"
FAMILY_SPREADSHEET = "Spreadsheet"
FAMILY_PRESENTATION = "Presentation"
FAMILY_DRAWING = "Drawing"

#---------------------#
# Configuration Start #
#---------------------#

# see http://wiki.services.openoffice.org/wiki/Framework/Article/Filter

# most formats are auto-detected; only those requiring options are defined here
IMPORT_FILTER_MAP = {
    "txt": {
        "FilterName": "Text (encoded)",
        "FilterOptions": "utf8"
    },
    "csv": {
        "FilterName": "Text - txt - csv (StarCalc)",
        "FilterOptions": "44,34,0"
    }
}

EXPORT_FILTER_MAP = {
    "pdf": {
        FAMILY_TEXT: { "FilterName": "writer_pdf_Export" },
        FAMILY_WEB: { "FilterName": "writer_web_pdf_Export" },
        FAMILY_SPREADSHEET: { "FilterName": "calc_pdf_Export" },
        FAMILY_PRESENTATION: { "FilterName": "impress_pdf_Export" },
        FAMILY_DRAWING: { "FilterName": "draw_pdf_Export" }
    },
    "html": {
        FAMILY_TEXT: { "FilterName": "HTML (StarWriter)" },
        FAMILY_SPREADSHEET: { "FilterName": "HTML (StarCalc)" },
        FAMILY_PRESENTATION: { "FilterName": "impress_html_Export" }
    },
    "odt": {
        FAMILY_TEXT: { "FilterName": "writer8" },
        FAMILY_WEB: { "FilterName": "writerweb8_writer" }
    },
    "doc": {
        FAMILY_TEXT: { "FilterName": "MS Word 97" }
    },
    "rtf": {
        FAMILY_TEXT: { "FilterName": "Rich Text Format" }
    },
    "txt": {
        FAMILY_TEXT: {
            "FilterName": "Text",
            "FilterOptions": "utf8"
        }
    },
    "ods": {
        FAMILY_SPREADSHEET: { "FilterName": "calc8" }
    },
    "xls": {
        FAMILY_SPREADSHEET: { "FilterName": "MS Excel 97" }
    },
    "csv": {
        FAMILY_SPREADSHEET: {
            "FilterName": "Text - txt - csv (StarCalc)",
            "FilterOptions": "44,34,0"
        }
    },
    "odp": {
        FAMILY_PRESENTATION: { "FilterName": "impress8" }
    },
    "ppt": {
        FAMILY_PRESENTATION: { "FilterName": "MS PowerPoint 97" }
    },
    "swf": {
        FAMILY_DRAWING: { "FilterName": "draw_flash_Export" },
        FAMILY_PRESENTATION: { "FilterName": "impress_flash_Export" }
    }
}

PAGE_STYLE_OVERRIDE_PROPERTIES = {
    FAMILY_SPREADSHEET: {
        #--- Scale options: uncomment 1 of the 3 ---
        # a) 'Reduce / enlarge printout': 'Scaling factor'
        "PageScale": 100,
        # b) 'Fit print range(s) to width / height': 'Width in pages' and 'Height in pages'
        #"ScaleToPagesX": 1, "ScaleToPagesY": 1000,
        # c) 'Fit print range(s) on number of pages': 'Fit print range(s) on number of pages'
        #"ScaleToPages": 1,
        "PrintGrid": False
    }
}

#-------------------#
# Configuration End #
#-------------------#

class OOService:
    """
    Start, stop, and connect to OpenOffice.
    """
    def __init__(self, port=DEFAULT_OPENOFFICE_PORT):
        """ Create OORunner that connects on the specified port. """
        self.port = port


    def connect(self, no_startup=False):
        """
        Connect to OpenOffice.
        If a connection cannot be established try to start OpenOffice.
        """
        localContext = uno.getComponentContext()
        resolver     = localContext.ServiceManager.createInstanceWithContext("com.sun.star.bridge.UnoUrlResolver", localContext)
        context      = None

        n = 0
        while n < 6:
            try:
                context = resolver.resolve("uno:socket,host=localhost,port=%d;urp;StarOffice.ComponentContext" % self.port)
                break
            except NoConnectException:
                pass

            # If first connect failed then try starting OpenOffice.
            if n == 0:
                # Exit loop if startup not desired.
                if no_startup:
                     break
                self.startup()

            # Pause and try again to connect
            time.sleep(1)
            n += 1

        if not context:
            raise Exception, "Failed to connect to OpenOffice on port %d" % self.port

        desktop = context.ServiceManager.createInstanceWithContext("com.sun.star.frame.Desktop", context)

        if not desktop:
            raise Exception, "Failed to create OpenOffice desktop on port %d" % self.port

        return desktop


    def startup(self):
        """
        Start a headless instance of OpenOffice.
        """
        args = [OPENOFFICE_BIN,
                '-accept=socket,host=localhost,port=%d;urp;StarOffice.ServiceManager' % self.port,
                '-norestore',
                '-nofirststartwizard',
                '-nologo',
                '-headless',
                ]
        env  = {'PATH'       : '/bin:/usr/bin:%s' % OPENOFFICE_PATH,
                'PYTHONPATH' : OPENOFFICE_LIBPATH,
                }

        try:
            pid = os.spawnve(os.P_NOWAIT, args[0], args, env)
        except Exception, e:
            raise Exception, "Failed to start OpenOffice on port %d: %s" % (self.port, e.message)

        if pid <= 0:
            raise Exception, "Failed to start OpenOffice on port %d" % self.port


    def shutdown(self):
        """
        Shutdown OpenOffice.
        """
        try:
            desktop = self.connect(True)
            if desktop:
                desktop.terminate()
        except Exception, e:
#            pass
            raise Exception, "Failed to shutdown the process: %s" % (e.message)




class DocumentConversionException(Exception):

    def __init__(self, message):
        self.message = message

    def __str__(self):
        return self.message


class DocumentConverter:
    
    def __init__(self, port=DEFAULT_OPENOFFICE_PORT):
        localContext = uno.getComponentContext()
        resolver = localContext.ServiceManager.createInstanceWithContext("com.sun.star.bridge.UnoUrlResolver", localContext)
        try:
            context = resolver.resolve("uno:socket,host=localhost,port=%s;urp;StarOffice.ComponentContext" % port)
        except NoConnectException:
            raise DocumentConversionException, "failed to connect to OpenOffice.org on port %s" % port
        self.desktop = context.ServiceManager.createInstanceWithContext("com.sun.star.frame.Desktop", context)

    def terminate(self):
	self.desktop.terminate()

    def convert(self, inputFile, outputFile):

        inputUrl = self._toFileUrl(inputFile)
        outputUrl = self._toFileUrl(outputFile)

        loadProperties = { "Hidden": True }
        inputExt = self._getFileExt(inputFile)
        if IMPORT_FILTER_MAP.has_key(inputExt):
            loadProperties.update(IMPORT_FILTER_MAP[inputExt])
        
        document = self.desktop.loadComponentFromURL(inputUrl, "_blank", 0, self._toProperties(loadProperties))
        try:
            document.refresh()
        except AttributeError:
            pass

        family = self._detectFamily(document)
        self._overridePageStyleProperties(document, family)
        
        outputExt = self._getFileExt(outputFile)
        storeProperties = self._getStoreProperties(document, outputExt)

        try:
            document.storeToURL(outputUrl, self._toProperties(storeProperties))
        finally:
            document.close(True)

    def _overridePageStyleProperties(self, document, family):
        if PAGE_STYLE_OVERRIDE_PROPERTIES.has_key(family):
            properties = PAGE_STYLE_OVERRIDE_PROPERTIES[family]
            pageStyles = document.getStyleFamilies().getByName('PageStyles')
            for styleName in pageStyles.getElementNames():
                pageStyle = pageStyles.getByName(styleName)
                for name, value in properties.items():
                    pageStyle.setPropertyValue(name, value)

    def _getStoreProperties(self, document, outputExt):
        family = self._detectFamily(document)
        try:
            propertiesByFamily = EXPORT_FILTER_MAP[outputExt]
        except KeyError:
            raise DocumentConversionException, "unknown output format: '%s'" % outputExt
        try:
            return propertiesByFamily[family]
        except KeyError:
            raise DocumentConversionException, "unsupported conversion: from '%s' to '%s'" % (family, outputExt)
    
    def _detectFamily(self, document):
        if document.supportsService("com.sun.star.text.WebDocument"):
            return FAMILY_WEB
        if document.supportsService("com.sun.star.text.GenericTextDocument"):
            # must be TextDocument or GlobalDocument
            return FAMILY_TEXT
        if document.supportsService("com.sun.star.sheet.SpreadsheetDocument"):
            return FAMILY_SPREADSHEET
        if document.supportsService("com.sun.star.presentation.PresentationDocument"):
            return FAMILY_PRESENTATION
        if document.supportsService("com.sun.star.drawing.DrawingDocument"):
            return FAMILY_DRAWING
        raise DocumentConversionException, "unknown document family: %s" % document

    def _getFileExt(self, path):
        ext = splitext(path)[1]
        if ext is not None:
            return ext[1:].lower()

    def _toFileUrl(self, path):
        return uno.systemPathToFileUrl(abspath(path))

    def _toProperties(self, dict):
        props = []
        for key in dict:
            prop = PropertyValue()
            prop.Name = key
            prop.Value = dict[key]
            props.append(prop)
        return tuple(props)


if __name__ == "__main__":
    from sys import argv, exit
    
    if argv[1] == "--daemon":
	try:
            service = OOService()
            service.startup()
	    exit(0)
	except ErrorCodeIOException, e:
	    print "Failed to start daemon process: %s" % e.message
	    exit(1)

    if argv[1] == "--shutdown":
	try:
            service = OOService()
            service.shutdown()
	    exit(0)
	except ErrorCodeIOException, e:
	    print "Failed to shut down daemon process: %s" % e.message
	    exit(1)

    if len(argv) < 3:
        print "USAGE: python %s <input-file> <output-file>" % argv[0]
        exit(255)
    elif not isfile(argv[1]):
        print "no such input file: %s" % argv[1]
        exit(1)
    try:
        converter = DocumentConverter()    
        converter.convert(argv[1], argv[2])
    except DocumentConversionException, exception:
        print "ERROR! " + str(exception)
        exit(1)
    except ErrorCodeIOException, exception:
        print "ERROR! ErrorCodeIOException %d" % exception.ErrCode
        exit(1)

