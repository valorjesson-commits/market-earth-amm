import unittest
from backend.models import app, db, UserProfile, sync_portfolio_holdings

class TestUserProfilePortfolio(unittest.TestCase):

    def setUp(self):
        # Configure app for testing with an in-memory SQLite database
        app.config["TESTING"] = True
        app.config["SQLALCHEMY_DATABASE_URI"] = "sqlite:///:memory:"
        self.app_context = app.app_context()
        self.app_context.push()
        db.create_all()

    def tearDown(self):
        db.session.remove()
        db.drop_all()
        self.app_context.pop()

    def test_default_user_profile_values(self):
        user = UserProfile(handle="test_user")
        db.session.add(user)
        db.session.commit()

        self.assertEqual(user.net_worth, 0.0)
        self.assertEqual(user.active_chains, "")
        self.assertFalse(user.is_verified)
        self.assertEqual(user.evm_balance, 0.0)
        self.assertEqual(user.icp_balance, 0.0)

        # Check serialization
        data = user.to_dict()
        self.assertEqual(data["handle"], "test_user")
        self.assertEqual(data["net_worth"], 0.0)
        self.assertEqual(data["active_chains"], "")
        self.assertFalse(data["is_verified"])

    def test_recalculate_net_worth_with_evm_only(self):
        user = UserProfile(handle="evm_user", evm_address="0x123f68493a")
        user.evm_balance = 1.5
        user.recalculate_net_worth()

        # 1.5 ETH * 3000 = 4500 USD
        self.assertEqual(user.net_worth, 4500.0)
        self.assertEqual(user.active_chains, "EVM")
        self.assertTrue(user.is_verified)

    def test_recalculate_net_worth_with_icp_only(self):
        user = UserProfile(handle="icp_user", icp_principal="aaaaa-aa")
        user.icp_balance = 50.0
        user.recalculate_net_worth()

        # 50.0 ICP * 10 = 500 USD
        self.assertEqual(user.net_worth, 500.0)
        self.assertEqual(user.active_chains, "ICP")
        self.assertTrue(user.is_verified)

    def test_recalculate_net_worth_with_both(self):
        user = UserProfile(handle="multi_user", evm_address="0x123", icp_principal="aaaaa-aa")
        user.evm_balance = 2.0
        user.icp_balance = 100.0
        user.recalculate_net_worth()

        # (2 * 3000) + (100 * 10) = 6000 + 1000 = 7000 USD
        self.assertEqual(user.net_worth, 7000.0)
        self.assertEqual(user.active_chains, "EVM,ICP")
        self.assertTrue(user.is_verified)

    def test_poll_and_update_balances_simulation(self):
        user = UserProfile(handle="poll_user", evm_address="0xabc", icp_principal="xyz")
        db.session.add(user)
        db.session.commit()

        # Test polling with default simulation behavior
        user.poll_and_update_balances()
        self.assertEqual(user.evm_balance, 1.25)
        self.assertEqual(user.icp_balance, 75.0)
        # (1.25 * 3000) + (75 * 10) = 3750 + 750 = 4500 USD
        self.assertEqual(user.net_worth, 4500.0)
        self.assertEqual(user.active_chains, "EVM,ICP")
        self.assertTrue(user.is_verified)

    def test_poll_and_update_balances_with_custom_mocks(self):
        user = UserProfile(handle="custom_user", evm_address="0x999", icp_principal="ppp")
        
        mock_evm = lambda addr: 10.0 if addr == "0x999" else 0.0
        mock_icp = lambda principal: 500.0 if principal == "ppp" else 0.0

        user.poll_and_update_balances(mock_evm_rpc=mock_evm, mock_ic_agent=mock_icp)
        self.assertEqual(user.evm_balance, 10.0)
        self.assertEqual(user.icp_balance, 500.0)
        # (10 * 3000) + (500 * 10) = 30000 + 5000 = 35000 USD
        self.assertEqual(user.net_worth, 35000.0)

    def test_sync_portfolio_holdings_service_function(self):
        user = UserProfile(handle="service_user", evm_address="0xdef", icp_principal="pqr")
        db.session.add(user)
        db.session.commit()

        success = sync_portfolio_holdings(user.id)
        self.assertTrue(success)

        # Refresh from database
        db.session.refresh(user)
        self.assertEqual(user.evm_balance, 1.25)
        self.assertEqual(user.icp_balance, 75.0)
        self.assertEqual(user.net_worth, 4500.0)
        self.assertTrue(user.is_verified)


if __name__ == "__main__":
    unittest.main()
