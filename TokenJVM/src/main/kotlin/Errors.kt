package com.snaulX.Tangara

class ImportError(line: Int, message: String) : TangaraError(line, message)

class InvalidNameError(line: Int, message: String) : TangaraError(line, message)

class SyntaxError(line: Int, message: String) : TangaraError(line, message)

class InvalidNumberError(line: Int, message: String) : TangaraError(line, message)

class InvalidCommandLineArgumentException(message: String?) : IllegalArgumentException(message)