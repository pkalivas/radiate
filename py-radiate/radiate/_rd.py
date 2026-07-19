import sys

import radiate.radiate as rd

rd.components = rd._constants.components
rd.loss_functions = rd._constants.loss_functions


sys.modules[__name__] = rd
