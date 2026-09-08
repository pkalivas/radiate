import sys

import radiate.radiate as rd

rd.components = rd._constants.components
rd.loss_functions = rd._constants.loss_functions
rd.event_types = rd._constants.event_types


sys.modules[__name__] = rd
