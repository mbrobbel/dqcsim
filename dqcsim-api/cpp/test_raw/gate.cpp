#include <dqcsim_raw.hpp>
#include "gtest/gtest.h"

using namespace dqcsim;

const double X_MATRIX[] = {
  0.0, 0.0,   1.0, 0.0,
  1.0, 0.0,   0.0, 0.0,
};

// Sanity check the gate API.
TEST(gate, sanity) {
  // Create handle.
  dqcs_handle_t a = dqcs_gate_new_custom("NOP", 0, 0, 0, NULL, 0);
  ASSERT_NE(a, 0) << "Unexpected error: " << dqcs_error_get();

  // Check that the handle is OK.
  EXPECT_EQ(dqcs_handle_type(a), dqcs_handle_type_t::DQCS_HTYPE_GATE);
  EXPECT_STREQ(dqcs_handle_dump(a), "Gate(\n    Gate {\n        name: Some(\n            \"NOP\"\n        ),\n        targets: [],\n        controls: [],\n        measures: [],\n        matrix: [],\n        data: ArbData {\n            json: Object(\n                {}\n            ),\n            args: []\n        }\n    }\n)");

  // Delete handle.
  EXPECT_EQ(dqcs_handle_delete(a), dqcs_return_t::DQCS_SUCCESS);

  // Check that the handle is no longer OK.
  EXPECT_EQ(dqcs_handle_type(a), dqcs_handle_type_t::DQCS_HTYPE_INVALID);
  EXPECT_STREQ(dqcs_handle_dump(a), nullptr);
  EXPECT_EQ(dqcs_error_get(), "Invalid argument: handle " + std::to_string(a) + " is invalid");
}

#define EXPECT_QBSET(qbset, ...) \
  do { \
    const dqcs_qubit_t qubits[] = {__VA_ARGS__, 0}; \
    expect_qbset(qbset, qubits); \
  } while (0)

void expect_qbset(dqcs_handle_t qbset, const dqcs_qubit_t *qubits) {
  int len = 0;
  EXPECT_NE(qbset, 0) << "Unexpected error: " << dqcs_error_get();
  if (qbset) {
    while (*qubits) {
      EXPECT_EQ(dqcs_qbset_contains(qbset, *qubits), dqcs_bool_return_t::DQCS_TRUE) << "Set does not contain qubit " << *qubits;
      qubits++;
      len++;
    }
    EXPECT_EQ(dqcs_qbset_len(qbset), len);
    EXPECT_EQ(dqcs_handle_delete(qbset), dqcs_return_t::DQCS_SUCCESS);
  }
}

#define EXPECT_MATRIX(gate, expected) expect_matrix(gate, expected, sizeof(expected) / 16);
#define EXPECT_NO_MATRIX(gate) expect_matrix(gate, NULL, 0);

void expect_matrix(dqcs_handle_t gate, const double *expected, int expected_len) {
  double *matrix = NULL;
  EXPECT_EQ(dqcs_gate_matrix_len(gate), expected_len);
  if (expected_len) {
    EXPECT_NE(matrix = dqcs_gate_matrix(gate), (double*)NULL) << "Unexpected error: " << dqcs_error_get();
    for (int i = 0; i < expected_len; i++) {
      EXPECT_EQ(matrix[i*2+0], expected[i*2+0]) << "matrix entry " << i << " real";
      EXPECT_EQ(matrix[i*2+1], expected[i*2+1]) << "matrix entry " << i << " imag";
    }
  } else {
    EXPECT_EQ(matrix = dqcs_gate_matrix(gate), (double*)NULL);
    EXPECT_STREQ(dqcs_error_get(), "Invalid argument: no matrix associated with gate");
  }
  if (matrix) free(matrix);
}

// Check X gate.
TEST(gate, x) {
  char *s;
  double *m;

  dqcs_handle_t targets = dqcs_qbset_new();
  ASSERT_NE(targets, 0) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_qbset_push(targets, 1), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();

  dqcs_handle_t a = dqcs_gate_new_unitary(targets, 0, X_MATRIX, 4);
  ASSERT_NE(a, 0) << "Unexpected error: " << dqcs_error_get();

  EXPECT_EQ(dqcs_gate_is_custom(a), dqcs_bool_return_t::DQCS_FALSE);
  EXPECT_STREQ(s = dqcs_gate_name(a), NULL);
  if (s) free(s);

  EXPECT_EQ(dqcs_gate_has_targets(a), dqcs_bool_return_t::DQCS_TRUE);
  EXPECT_QBSET(dqcs_gate_targets(a), 1);

  EXPECT_EQ(dqcs_gate_has_controls(a), dqcs_bool_return_t::DQCS_FALSE);
  EXPECT_QBSET(dqcs_gate_controls(a), 0);

  EXPECT_EQ(dqcs_gate_has_measures(a), dqcs_bool_return_t::DQCS_FALSE);
  EXPECT_QBSET(dqcs_gate_measures(a), 0);

  EXPECT_MATRIX(a, X_MATRIX);

  EXPECT_STREQ(s = dqcs_arb_json_get(a), "{}");
  if (s) free(s);
  EXPECT_EQ(dqcs_arb_len(a), 0);

  EXPECT_EQ(dqcs_handle_delete(a), dqcs_return_t::DQCS_SUCCESS);
}

// Check CNOT gate.
TEST(gate, cnot) {
  char *s;
  double *m;

  dqcs_handle_t targets = dqcs_qbset_new();
  ASSERT_NE(targets, 0) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_qbset_push(targets, 1), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();

  dqcs_handle_t controls = dqcs_qbset_new();
  ASSERT_NE(controls, 0) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_qbset_push(controls, 2), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();

  dqcs_handle_t a = dqcs_gate_new_unitary(targets, controls, X_MATRIX, 4);
  ASSERT_NE(a, 0) << "Unexpected error: " << dqcs_error_get();

  EXPECT_EQ(dqcs_gate_is_custom(a), dqcs_bool_return_t::DQCS_FALSE);
  EXPECT_STREQ(s = dqcs_gate_name(a), NULL);
  if (s) free(s);

  EXPECT_EQ(dqcs_gate_has_targets(a), dqcs_bool_return_t::DQCS_TRUE);
  EXPECT_QBSET(dqcs_gate_targets(a), 1);

  EXPECT_EQ(dqcs_gate_has_controls(a), dqcs_bool_return_t::DQCS_TRUE);
  EXPECT_QBSET(dqcs_gate_controls(a), 2);

  EXPECT_EQ(dqcs_gate_has_measures(a), dqcs_bool_return_t::DQCS_FALSE);
  EXPECT_QBSET(dqcs_gate_measures(a), 0);

  EXPECT_MATRIX(a, X_MATRIX);

  EXPECT_STREQ(s = dqcs_arb_json_get(a), "{}");
  if (s) free(s);
  EXPECT_EQ(dqcs_arb_len(a), 0);

  EXPECT_EQ(dqcs_handle_delete(a), dqcs_return_t::DQCS_SUCCESS);
}

// Check measure gate.
TEST(gate, measure) {
  char *s;
  double *m;

  dqcs_handle_t measures = dqcs_qbset_new();
  ASSERT_NE(measures, 0) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_qbset_push(measures, 1), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_qbset_push(measures, 2), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();

  dqcs_handle_t a = dqcs_gate_new_measurement(measures);
  ASSERT_NE(a, 0) << "Unexpected error: " << dqcs_error_get();

  EXPECT_EQ(dqcs_gate_is_custom(a), dqcs_bool_return_t::DQCS_FALSE);
  EXPECT_STREQ(s = dqcs_gate_name(a), NULL);
  if (s) free(s);

  EXPECT_EQ(dqcs_gate_has_targets(a), dqcs_bool_return_t::DQCS_FALSE);
  EXPECT_QBSET(dqcs_gate_targets(a), 0);

  EXPECT_EQ(dqcs_gate_has_controls(a), dqcs_bool_return_t::DQCS_FALSE);
  EXPECT_QBSET(dqcs_gate_controls(a), 0);

  EXPECT_EQ(dqcs_gate_has_measures(a), dqcs_bool_return_t::DQCS_TRUE);
  EXPECT_QBSET(dqcs_gate_measures(a), 1, 2);

  EXPECT_NO_MATRIX(a);

  EXPECT_STREQ(s = dqcs_arb_json_get(a), "{}");
  if (s) free(s);
  EXPECT_EQ(dqcs_arb_len(a), 0);

  EXPECT_EQ(dqcs_handle_delete(a), dqcs_return_t::DQCS_SUCCESS);
}

// Check NOP custom gate.
TEST(gate, nop) {
  char *s;
  double *m;

  dqcs_handle_t a = dqcs_gate_new_custom("NOP", 0, 0, 0, NULL, 0);
  ASSERT_NE(a, 0) << "Unexpected error: " << dqcs_error_get();

  EXPECT_EQ(dqcs_gate_is_custom(a), dqcs_bool_return_t::DQCS_TRUE);
  EXPECT_STREQ(s = dqcs_gate_name(a), "NOP");
  if (s) free(s);

  EXPECT_EQ(dqcs_gate_has_targets(a), dqcs_bool_return_t::DQCS_FALSE);
  EXPECT_QBSET(dqcs_gate_targets(a), 0);

  EXPECT_EQ(dqcs_gate_has_controls(a), dqcs_bool_return_t::DQCS_FALSE);
  EXPECT_QBSET(dqcs_gate_controls(a), 0);

  EXPECT_EQ(dqcs_gate_has_measures(a), dqcs_bool_return_t::DQCS_FALSE);
  EXPECT_QBSET(dqcs_gate_measures(a), 0);

  EXPECT_NO_MATRIX(a);

  EXPECT_STREQ(s = dqcs_arb_json_get(a), "{}");
  if (s) free(s);
  EXPECT_EQ(dqcs_arb_len(a), 0);

  EXPECT_EQ(dqcs_handle_delete(a), dqcs_return_t::DQCS_SUCCESS);
}

// Check complex custom gate.
TEST(gate, discombobulate) {
  char *s;
  double *m;

  dqcs_handle_t targets = dqcs_qbset_new();
  ASSERT_NE(targets, 0) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_qbset_push(targets, 1), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();

  dqcs_handle_t controls = dqcs_qbset_new();
  ASSERT_NE(controls, 0) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_qbset_push(controls, 2), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();

  dqcs_handle_t measures = dqcs_qbset_new();
  ASSERT_NE(measures, 0) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_qbset_push(measures, 1), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_qbset_push(measures, 2), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();

  dqcs_handle_t a = dqcs_gate_new_custom("DISCOMBOBULATE", targets, controls, measures, X_MATRIX, 4);
  ASSERT_NE(a, 0) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_arb_json_set(a, "{\"sequence\": [4, 8, 15, 16, 23, 42]}"), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_arb_push_str(a, "(%@#(*^"), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();

  EXPECT_EQ(dqcs_gate_is_custom(a), dqcs_bool_return_t::DQCS_TRUE);
  EXPECT_STREQ(s = dqcs_gate_name(a), "DISCOMBOBULATE");
  if (s) free(s);

  EXPECT_EQ(dqcs_gate_has_targets(a), dqcs_bool_return_t::DQCS_TRUE);
  EXPECT_QBSET(dqcs_gate_targets(a), 1);

  EXPECT_EQ(dqcs_gate_has_controls(a), dqcs_bool_return_t::DQCS_TRUE);
  EXPECT_QBSET(dqcs_gate_controls(a), 2);

  EXPECT_EQ(dqcs_gate_has_measures(a), dqcs_bool_return_t::DQCS_TRUE);
  EXPECT_QBSET(dqcs_gate_measures(a), 1, 2);

  EXPECT_MATRIX(a, X_MATRIX);

  EXPECT_STREQ(s = dqcs_arb_json_get(a), "{\"sequence\":[4,8,15,16,23,42]}");
  if (s) free(s);
  EXPECT_EQ(dqcs_arb_len(a), 1);

  EXPECT_EQ(dqcs_handle_delete(a), dqcs_return_t::DQCS_SUCCESS);
}

// Check disallowed gates.
TEST(gate, erroneous) {
  char *s;
  double *m;

  dqcs_handle_t qbset_a = dqcs_qbset_new();
  ASSERT_NE(qbset_a, 0) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_qbset_push(qbset_a, 1), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();

  dqcs_handle_t qbset_b = dqcs_qbset_new();
  ASSERT_NE(qbset_b, 0) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_qbset_push(qbset_b, 1), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_qbset_push(qbset_b, 2), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_qbset_push(qbset_b, 3), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();

  dqcs_handle_t qbset_c = dqcs_qbset_new();
  ASSERT_NE(qbset_c, 0) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_qbset_push(qbset_c, 6), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();
  EXPECT_EQ(dqcs_qbset_push(qbset_c, 7), dqcs_return_t::DQCS_SUCCESS) << "Unexpected error: " << dqcs_error_get();

  // Invalid unitaries.
  EXPECT_EQ(dqcs_gate_new_unitary(0, 0, NULL, 0), 0);
  EXPECT_STREQ(dqcs_error_get(), "Invalid argument: handle 0 is invalid");

  EXPECT_EQ(dqcs_gate_new_unitary(qbset_a, qbset_b, X_MATRIX, 4), 0);
  EXPECT_STREQ(dqcs_error_get(), "Invalid argument: qubit 1 is used more than once");

  EXPECT_EQ(dqcs_gate_new_unitary(qbset_b, 0, X_MATRIX, 4), 0);
  EXPECT_STREQ(dqcs_error_get(), "Invalid argument: matrix has the wrong number of entries");

  EXPECT_EQ(dqcs_gate_new_unitary(qbset_a, 0, NULL, 4), 0);
  EXPECT_STREQ(dqcs_error_get(), "Invalid argument: matrix pointer is null");

  EXPECT_EQ(dqcs_gate_new_unitary(qbset_a, 0, NULL, 0), 0);
  EXPECT_STREQ(dqcs_error_get(), "Invalid argument: the unitary matrix cannot be null");

  // Invalid measures.
  EXPECT_EQ(dqcs_gate_new_measurement(0), 0);
  EXPECT_STREQ(dqcs_error_get(), "Invalid argument: handle 0 is invalid");

  // Invalid custom gates.
  ASSERT_EQ(dqcs_gate_new_custom(NULL, 0, 0, 0, NULL, 0), 0);
  EXPECT_STREQ(dqcs_error_get(), "Invalid argument: unexpected NULL string");

  ASSERT_EQ(dqcs_gate_new_custom("FOO", qbset_a, qbset_b, qbset_c, NULL, 0), 0);
  EXPECT_STREQ(dqcs_error_get(), "Invalid argument: qubit 1 is used more than once");

  EXPECT_EQ(dqcs_gate_new_custom("BAR", qbset_b, qbset_c, qbset_a, X_MATRIX, 4), 0);
  EXPECT_STREQ(dqcs_error_get(), "Invalid argument: matrix has the wrong number of entries");
}