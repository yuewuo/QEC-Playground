
for error in ["X", "Z", "Y", "E"]:
    print(f"""
def _get_p{error}(self):
    return self.source.error_model["snapshot"][self._t][self._i][self._j]['error_rate_{error.lower()}']
def _set_p{error}(self, error_rate):
    self.source.error_model["snapshot"][self._t][self._i][self._j]['error_rate_{error.lower()}'] = error_rate
p{error} = property(
    fset=_set_p{error},
    fget=_get_p{error},
    doc="single-qubit error rate of Pauli {error}"
)""") 


print("")

correlated_errors = []
for e1 in ["I", "X", "Z", "Y"]:
    for e2 in ["I", "X", "Z", "Y"]:
        correlated_errors.append(f"{e1}{e2}")
correlated_errors = correlated_errors[1:]
print(correlated_errors)

for error in correlated_errors:
    print(f"""
def _get_p{error}(self):
    node = self.source.error_model["snapshot"][self._t][self._i][self._j]
    if node["connection"] is None:
        raise Exception(f"qubit[{{self._t}}][{{self._i}}][{{self._j}}] doesn't have two-qubit gate here, error rate of {error} is invalid")
    return node['correlated_error_model']['error_rate_{error.lower()}']
def _set_p{error}(self, error_rate):
    node = self.source.error_model["snapshot"][self._t][self._i][self._j]
    if node["connection"] is None:
        raise Exception(f"qubit[{{self._t}}][{{self._i}}][{{self._j}}] doesn't have two-qubit gate here, error rate of {error} is invalid")
    node['correlated_error_model']['error_rate_{error.lower()}'] = error_rate
p{error} = property(
    fset=_set_p{error},
    fget=_get_p{error},
    doc="two-qubit error rate of Pauli {error}"
)""") 



for error in correlated_errors:
    print(f"qubit.p{error} = 0")

for error in ["IE", "EI", "EE"]:
    print(f"""
def _get_p{error}(self):
    node = self.source.error_model["snapshot"][self._t][self._i][self._j]
    if node["connection"] is None:
        raise Exception(f"qubit[{{self._t}}][{{self._i}}][{{self._j}}] doesn't have two-qubit gate here, error rate of {error} is invalid")
    return node['correlated_erasure_error_model']['error_rate_{error.lower()}']
def _set_p{error}(self, error_rate):
    node = self.source.error_model["snapshot"][self._t][self._i][self._j]
    if node["connection"] is None:
        raise Exception(f"qubit[{{self._t}}][{{self._i}}][{{self._j}}] doesn't have two-qubit gate here, error rate of {error} is invalid")
    node['correlated_erasure_error_model']['error_rate_{error.lower()}'] = error_rate
p{error} = property(
    fset=_set_p{error},
    fget=_get_p{error},
    doc="two-qubit error rate of Pauli {error}"
)""") 
